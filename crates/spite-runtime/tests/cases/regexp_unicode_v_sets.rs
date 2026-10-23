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
        (r"[[a--a]&&[\q{ab}]]", false),
        (r"[\q{}&&a]", false),
        (r"[a--a--\q{ab}]", false),
        (r"[a&&b&&\q{ab}]", false),
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
        r"[\p{RGI_Emoji}&&a]",
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
    check(r"/\b[[\q{a}]]{0}\b/v.test('a')");
    for source in [
        r"/\b^[[a]]+/v.test('a')",
        r"/\b[[a]]+$\b/v.test('a')",
        r"/(\b[[a]]\b)+/v.test('a')",
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
    assert_eq!(
        Realm::default().eval(r"/[a--a--\q{a}]c/v.test('ac')"),
        Ok(Value::Boolean(false))
    );
    for source in [
        r"/[a--b]+c/v.test('ac')",
        r"/[a--b]cd/v.test('acd')",
        r"/([a--b])c/v.test('ac')",
        r"/[a--b](c)/v.test('ac')",
        r"/([a--b]c)+/v.test('ac')",
        r"/[a--b]^c/v.test('ac')",
        r"/[a--b]$c/v.test('ac')",
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
    check(r"!/c[a--a--\q{a}]/v.test('ca')");
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
    check(r"/c[a--b][c]/v.test('cac')");
    check(
        r#"let r=/c[a--b][c]/v,m=r.exec('cac');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cac'&&m.index===0&&m.input==='cac'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[a--b][c]'&&r.lastIndex===0&&!r.test('xac')&&!r.test('cxc')&&!r.test('cax')&&!r.test('ca')&&r.test('cacx')&&r.exec('xcac').index===1"#,
    );
    for source in [
        r"/cc[a--b]/v.test('cca')",
        r"/c[a--b]+/v.test('ca')",
        r"/(c)[a--b]/v.test('ca')",
        r"/c([a--b])/v.test('ca')",
        r"/(c[a--b])+/v.test('ca')",
        r"/c^[a--b]/v.test('ca')",
        r"/c$[a--b]/v.test('ca')",
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
    check(r"/\ba[[a]][c]/v.test('aac')");
    check(
        r#"let r=/\ba[[a]][c]/v,m=r.exec('aac');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='aac'&&m.index===0&&m.input==='aac'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\ba[[a]][c]'&&r.lastIndex===0&&!r.test('xac')&&!r.test('axc')&&!r.test('aax')&&!r.test('aa')&&r.test('aacx')&&!r.test('xaac')&&r.exec('!aac').index===1"#,
    );
    assert_eq!(
        Realm::default().eval(r"/\b[[a--a]&&[\q{a}]]c/v.test('ac')"),
        Ok(Value::Boolean(false))
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
    check(r"/c[a--b][c]/v.test('cac')");
    check(
        r#"let r=/c[a--b][c]/v,m=r.exec('cac');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cac'&&m.index===0&&m.input==='cac'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[a--b][c]'&&r.lastIndex===0&&!r.test('xac')&&!r.test('cxc')&&!r.test('cax')&&!r.test('ca')&&r.test('cacx')&&r.exec('xcac').index===1"#,
    );
    for source in [
        r"/[[a]]\b[[b]]/v.test('ab')",
        r"/[[a]]^[[b]]/v.test('ab')",
        r"/([[a]])[[b]]/v.test('ab')",
        r"/[[a]]([[b]])/v.test('ab')",
        r"/[[a]]+[[b]]/v.test('ab')",
        r"/[[a]][[b]]?/v.test('ab')",
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

    // Exact original Boolean controls now complete through checked-prefix/q-suffix proof.
    assert_eq!(
        Realm::default().eval(r"/[a--a][[\q{b}]]/v.test('ab')"),
        Ok(Value::Boolean(false))
    );
    check(
        r#"let r=/(?<whole>([a--a][[\q{b}]]))/dvg;r.lastIndex=1;let m=r.exec("ab");m===null&&r.lastIndex===0&&r.source==="(?<whole>([a--a][[\\q{b}]]))"&&r.flags==='dgv'"#,
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
    check(r"/\b[[a]][[b]][[c]]/v.test('abc')");
    check(
        r"let r=/\b[[a]][[b]][[c]]/v,m=r.exec('abc');m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]][[b]][[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')",
    );
    for source in [
        r"/[[a]]\b[[b]]/v.test('ab')",
        r"/\b^[[a]][[b]]/v.test('ab')",
        r"/[[a]][[b]]$\b/v.test('ab')",
        r"/\b\b[[a]][[b]]/v.test('ab')",
        r"/(\b[[a]])[[b]]/v.test('ab')",
        r"/\b[[a]]([[b]])/v.test('ab')",
        r"/\b[[a]]+[[b]]/v.test('ab')",
        r"/\b[[a]][[b]]?/v.test('ab')",
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

    // Exact original Boolean controls now complete through checked-prefix/q-suffix proof.
    assert_eq!(
        Realm::default().eval(r"/\b[a--a][[\q{b}]]/v.test('ab')"),
        Ok(Value::Boolean(false))
    );
    check(
        r#"let r=/(?<whole>(\b[a--a][[\q{b}]]))/dvg;r.lastIndex=1;let m=r.exec("ab");m===null&&r.lastIndex===0&&r.source==="(?<whole>(\\b[a--a][[\\q{b}]]))"&&r.flags==='dgv'"#,
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
    check(r"/[a--b][c][d]/v.test('acd')");
    check(
        r"let r=/[a--b][c][d]/v,m=r.exec('acd');m!==null&&m.length===1&&m[0]==='acd'&&m.index===0&&m.input==='acd'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a--b][c][d]'&&r.lastIndex===0&&!r.test('xcd')&&!r.test('axd')&&!r.test('acx')&&!r.test('ac')&&r.exec('xacd').index===1&&r.exec('acdx').index===0&&r.lastIndex===0",
    );
    check(r"/c[a--b][c]/v.test('cac')");
    check(
        r#"let r=/c[a--b][c]/v,m=r.exec('cac');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cac'&&m.index===0&&m.input==='cac'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[a--b][c]'&&r.lastIndex===0&&!r.test('xac')&&!r.test('cxc')&&!r.test('cax')&&!r.test('ca')&&r.test('cacx')&&r.exec('xcac').index===1"#,
    );
    for source in [
        r"/[a--b]^[c]/v.test('ac')",
        r"/([a--b])[c]/v.test('ac')",
        r"/[a--b]([c])/v.test('ac')",
        r"/[a--b]+[c]/v.test('ac')",
        r"/[a--b][c]?/v.test('ac')",
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

    // Exact original Boolean controls now complete through checked-prefix/q-suffix proof.
    assert_eq!(
        Realm::default().eval(r"/[a--a][\q{c}]/v.test('ac')"),
        Ok(Value::Boolean(false))
    );
    check(
        r#"let r=/(?<whole>([a--a][\q{c}]))/dvg;r.lastIndex=1;let m=r.exec("ac");m===null&&r.lastIndex===0&&r.source==="(?<whole>([a--a][\\q{c}]))"&&r.flags==='dgv'"#,
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
    check(r"/[a][c--b][d]/v.test('acd')");
    check(
        r"let r=/[a][c--b][d]/v,m=r.exec('acd');m!==null&&m.length===1&&m[0]==='acd'&&m.index===0&&m.input==='acd'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a][c--b][d]'&&r.lastIndex===0&&!r.test('xcd')&&!r.test('axd')&&!r.test('acx')&&!r.test('ac')&&r.exec('xacd').index===1&&r.exec('acdx').index===0&&r.lastIndex===0",
    );
    check(r"/c[a][c--b]/v.test('cac')");
    check(
        r#"let r=/c[a][c--b]/v,m=r.exec('cac');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cac'&&m.index===0&&m.input==='cac'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[a][c--b]'&&r.lastIndex===0&&!r.test('xac')&&!r.test('cxc')&&!r.test('cax')&&!r.test('ca')&&r.test('cacx')&&r.exec('xcac').index===1"#,
    );
    assert_eq!(
        Realm::default().eval(r"/[][[\q{c}]]/v.test('ac')"),
        Ok(Value::Boolean(false))
    );
    for source in [
        r"/[a]^[b--c]/v.test('ac')",
        r"/([a])[b--c]/v.test('ac')",
        r"/[a]([b--c])/v.test('ac')",
        r"/[a]+[b--c]/v.test('ac')",
        r"/[a][c--b]?/v.test('ac')",
        r"/[][[\p{RGI_Emoji}]]/v.test('ac')",
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
    check(r"/\b[[a]][b][[c]]/v.test('abc')");
    check(
        r"let r=/\b[[a]][b][[c]]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]][b][[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')&&r.exec('!abc').index===1&&r.exec('abcx').index===0&&r.exec('abcx')[0]==='abc'&&r.lastIndex===0",
    );
    assert_eq!(
        Realm::default().eval(r"/\b[[\q{a}]][]/v.test('ab')"),
        Ok(Value::Boolean(false))
    );
    for source in [
        r"/[[a]]\b[b]/v.test('ab')",
        r"/\b^[[a]][b]/v.test('ab')",
        r"/[[a]][b]$\b/v.test('ab')",
        r"/\b\b[[a]][b]/v.test('ab')",
        r"/(\b[[a]])[b]/v.test('ab')",
        r"/\b[[a]]([b])/v.test('ab')",
        r"/\b[[a]]+[b]/v.test('ab')",
        r"/\b[[a]][b]?/v.test('ab')",
        r"/\b[a--a][\p{RGI_Emoji}]/v.test('ab')",
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

    // Exact original Boolean controls now complete through checked-prefix/q-suffix proof.
    assert_eq!(
        Realm::default().eval(r"/\b[a--a][\q{b}]/v.test('ab')"),
        Ok(Value::Boolean(false))
    );
    check(
        r#"let r=/(?<whole>(\b[a--a][\q{b}]))/dvg;r.lastIndex=1;let m=r.exec("ab");m===null&&r.lastIndex===0&&r.source==="(?<whole>(\\b[a--a][\\q{b}]))"&&r.flags==='dgv'"#,
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
    check(r"/\b[a][[b]][c]/v.test('abc')");
    check(
        r"let r=/\b[a][[b]][c]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[a][[b]][c]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')&&r.exec('!abc').index===1&&r.exec('abcx').index===0&&r.exec('abcx')[0]==='abc'&&r.lastIndex===0",
    );
    assert_eq!(
        Realm::default().eval(r"/\b[][[\q{b}]]/v.test('ab')"),
        Ok(Value::Boolean(false))
    );
    for source in [
        r"/[a]\b[[b]]/v.test('ab')",
        r"/\b^[a][[b]]/v.test('ab')",
        r"/[a][[b]]$\b/v.test('ab')",
        r"/\b\b[a][[b]]/v.test('ab')",
        r"/(\b[a])[[b]]/v.test('ab')",
        r"/\b[a]([[b]])/v.test('ab')",
        r"/\b[a]+[[b]]/v.test('ab')",
        r"/\b[a][[b]]?/v.test('ab')",
        r"/\b[][[\p{RGI_Emoji}]]/v.test('ab')",
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
    check(r"/[a][c][d]/v.test('acd')");
    check(
        r"let r=/[a][c][d]/v,m=r.exec('acd');m!==null&&m.length===1&&m[0]==='acd'&&m.index===0&&m.input==='acd'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a][c][d]'&&r.lastIndex===0&&!r.test('xcd')&&!r.test('axd')&&!r.test('acx')&&!r.test('ac')&&r.exec('xacd').index===1&&r.exec('acdx').index===0",
    );
    check(r"/c[a][c]/v.test('cac')");
    check(
        r#"let r=/c[a][c]/v,m=r.exec('cac');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cac'&&m.index===0&&m.input==='cac'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[a][c]'&&r.lastIndex===0&&!r.test('xac')&&!r.test('cxc')&&!r.test('cax')&&!r.test('ca')&&r.test('xcac')&&r.test('cacx')"#,
    );
    assert_eq!(
        Realm::default().eval(r"/[\q{a}][]/v.test('ab')"),
        Ok(Value::Boolean(false))
    );
    assert_eq!(
        Realm::default().eval(r"/[][\q{c}]/v.test('ac')"),
        Ok(Value::Boolean(false))
    );
    for source in [
        r"/[a]^[c]/v.test('ac')",
        r"/([a])[c]/v.test('ac')",
        r"/[a]([c])/v.test('ac')",
        r"/[a]+[c]/v.test('ac')",
        r"/[a][c]?/v.test('ac')",
        r"/[][\p{RGI_Emoji}]/v.test('ac')",
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
    check(r"/\b[a][b][c]/v.test('abc')");
    check(
        r"let r=/\b[a][b][c]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[a][b][c]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')&&r.exec('!abc').index===1&&r.exec('abcx').index===0&&r.exec('abcx')[0]==='abc'",
    );
    assert_eq!(
        Realm::default().eval(r"/\b[\q{a}][]/v.test('ab')"),
        Ok(Value::Boolean(false))
    );
    assert_eq!(
        Realm::default().eval(r"/\b[][\q{b}]/v.test('ab')"),
        Ok(Value::Boolean(false))
    );
    for source in [
        r"/[a]\b[b]/v.test('ab')",
        r"/\b^[a][b]/v.test('ab')",
        r"/[a][b]$\b/v.test('ab')",
        r"/\b\b[a][b]/v.test('ab')",
        r"/(\b[a])[b]/v.test('ab')",
        r"/\b[a]([b])/v.test('ab')",
        r"/\b[a]+[b]/v.test('ab')",
        r"/\b[a][b]?/v.test('ab')",
        r"/\b[][\p{RGI_Emoji}]/v.test('ab')",
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
    check(r"/\bc[[a]][[b]]/v.test('cab')");
    check(
        r"let r=/\bc[[a]][[b]]/v,m=r.exec('cab');m!==null&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\bc[[a]][[b]]'&&r.lastIndex===0&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&!r.test('xcab')&&r.test('cabx')",
    );
    check(r"/c[[a]][[b]]\b/v.test('cab')");
    check(
        r"let r=/c[[a]][[b]]\b/v,m=r.exec('cab');m!==null&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[[a]][[b]]\\b'&&r.lastIndex===0&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&!r.test('cabx')&&r.test('xcab')",
    );
    check(r"/c[a][b]/v.test('cab')");
    check(
        r#"let r=/c[a][b]/v,m=r.exec('cab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[a][b]'&&r.lastIndex===0&&!r.test('xab')&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&r.test('xcab')&&r.test('cabx')"#,
    );
    check(r"/c[[a]][b]/v.test('cab')");
    check(
        r#"let r=/c[[a]][b]/v,m=r.exec('cab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[[a]][b]'&&r.lastIndex===0&&!r.test('xab')&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&r.test('cabx')&&r.exec('xcab').index===1"#,
    );
    check(r"/c[a][[b]]/v.test('cab')");
    check(
        r#"let r=/c[a][[b]]/v,m=r.exec('cab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[a][[b]]'&&r.lastIndex===0&&!r.test('xab')&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&r.test('cabx')&&r.exec('xcab').index===1"#,
    );
    for source in [
        r"/ca[[a]][[b]]/v.test('caab')",
        r"/c[[a]][[b]][[c]]/v.test('cabc')",
        r"/c[[a]]x[[b]]/v.test('caxb')",
        r"/c[[a]]\b[[b]]/v.test('cab')",
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
    check(r"/\b[[a]][[b]]c/v.test('abc')");
    check(
        r"let r=/\b[[a]][[b]]c/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]][[b]]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')&&r.test('abcx')",
    );
    check(r"/[[a]][[b]]c\b/v.test('abc')");
    check(
        r"let r=/[[a]][[b]]c\b/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][[b]]c\\b'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('abcx')&&r.test('xabc')",
    );
    check(r"/[[a]][b]c/v.test('abc')");
    check(
        r#"let r=/[[a]][b]c/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][b]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc').index===1"#,
    );
    check(r"/[a][[b]]c/v.test('abc')");
    check(
        r#"let r=/[a][[b]]c/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a][[b]]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc').index===1"#,
    );
    for source in [
        r"/[[a]][[b]]cd/v.test('abcd')",
        r"/[[a]][[b]]^c/v.test('abc')",
        r"/[[a]]x[[b]]c/v.test('axbc')",
        r"/[[a]][[b]]\bc/v.test('abc')",
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
    check(r"/\b[[a]]b[[c]]/v.test('abc')");
    check(
        r"let r=/\b[[a]]b[[c]]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]]b[[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')&&r.test('abcx')",
    );
    check(r"/[[a]]b[[c]]\b/v.test('abc')");
    check(
        r"let r=/[[a]]b[[c]]\b/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]]b[[c]]\\b'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('abcx')&&r.test('xabc')",
    );
    check(r"/[[a]][b][[c]]/v.test('abc')");
    check(
        r"let r=/[[a]][b][[c]]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][b][[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('xabc').index===1&&r.exec('abcx').index===0&&r.lastIndex===0",
    );
    check(r"/[[a]]b[c]/v.test('abc')");
    check(
        r#"let r=/[[a]]b[c]/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]]b[c]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc').index===1"#,
    );
    check(r"/[a]b[[c]]/v.test('abc')");
    check(
        r#"let r=/[a]b[[c]]/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a]b[[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc').index===1"#,
    );
    for source in [
        r"/a[[a]]b[[c]]/v.test('aabc')",
        r"/[[a]]b[[c]]d/v.test('abcd')",
        r"/[[a]]bc[[d]]/v.test('abcd')",
        r"/[[a]]^b[[c]]/v.test('abc')",
        r"/[[a]]\bb[[c]]/v.test('abc')",
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
    check(r"/\b[[a]][[b]][[c]]/v.test('abc')");
    check(
        r"let r=/\b[[a]][[b]][[c]]/v,m=r.exec('abc');m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]][[b]][[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')",
    );
    check(r"/[[a]][[b]][[c]]\b/v.test('abc')");
    check(
        r"let r=/[[a]][[b]][[c]]\b/v,m=r.exec('abc');m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][[b]][[c]]\\b'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('abcx')",
    );
    check(r"/[[a]][[b]][[c]][[d]]/v.test('abcd')");
    check(
        r"let r=/[[a]][[b]][[c]][[d]]/v,m=r.exec('abcd');m!==null&&m.length===1&&m[0]==='abcd'&&m.index===0&&m.input==='abcd'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][[b]][[c]][[d]]'&&r.lastIndex===0&&!r.test('xbcd')&&!r.test('axcd')&&!r.test('abxd')&&!r.test('abcx')&&!r.test('abc')&&r.exec('xabcd').index===1&&r.exec('abcdx').index===0",
    );
    check(r"/[a][[b]][[c]]/v.test('abc')");
    check(
        r"let r=/[a][[b]][[c]]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a][[b]][[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('xabc').index===1&&r.exec('abcx').index===0&&r.lastIndex===0",
    );
    check(r"/[[a]][b][[c]]/v.test('abc')");
    check(
        r"let r=/[[a]][b][[c]]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][b][[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('xabc').index===1&&r.exec('abcx').index===0&&r.lastIndex===0",
    );
    check(r"/[[a]][[b]][c]/v.test('abc')");
    check(
        r"let r=/[[a]][[b]][c]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][[b]][c]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('xabc').index===1&&r.exec('abcx').index===0&&r.lastIndex===0",
    );
    for source in [
        r"/x[[a]][[b]][[c]]/v.test('xabc')",
        r"/[[a]][[b]][[c]]x/v.test('abcx')",
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

#[test]
fn checked_v_set_class_triple_word_state_context_and_consumers() {
    check(
        r"/\b[a--b][[c]][[d]]\b/v.test('acd')&&!/\b[a--b][[c]][[d]]\b/v.test('bcd')&&!/\b[a--b][[c]][[d]]\b/v.test('ac')&&!/\b[a--a][[c]][[b]]\b/v.test('acb')&&!/\b[[a]][c--c][[b]]\b/v.test('acb')&&!/\b[[a]][[c]][b--b]\b/v.test('acb')",
    );
    check(r#"let marker=0;eval("marker=1; /\\b[a--b][[c]][[d]]\\b/v.test('acd')")&&marker===1"#);
    check(
        r"let a=/[[a]][[a]][[a]]\b/dvg,b=/\B[[a]][[a]][[a]]/dvg,x=a.exec('aaaa!'),y=b.exec('aaaa!');x.index===1&&x[0]==='aaa'&&x.indices[0][1]===4&&a.lastIndex===4&&y.index===1&&y[0]==='aaa'&&y.indices[0][1]===4&&b.lastIndex===4",
    );
    check(
        r"let g=/\B[[a]][[a]][[a]]\b/vg,y=/\B[[a]][[a]][[a]]\b/vy,a=g.exec('aaaa!');a.index===1&&a[0]==='aaa'&&g.lastIndex===4&&g.exec('aaaa!')===null&&g.lastIndex===0&&y.exec('aaaa!')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaaa!')[0]==='aaa')&&y.lastIndex===4",
    );
    check(
        r"/\b[[a]][[b]][[c]]/v.test('abc')&&/\B[[a]][[b]][[c]]/v.test('xabc')&&/[[a]][[b]][[c]]\b/v.test('abc!')&&/[[a]][[b]][[c]]\B/v.test('abcx')&&/\b[[a]][[b]][[c]]\b/v.test('!abc!')&&/\b[[a]][[b]][[c]]\B/v.test('!abcx')&&/\B[[a]][[b]][[c]]\b/v.test('xabc!')&&/\B[[a]][[b]][[c]]\B/v.test('xabcx')",
    );
    check(
        r"!/\b[[a]][[b]][[c]]/v.test('xabc')&&!/[[a]][[b]][[c]]\b/v.test('abcx')&&!/\B[[a]][[b]][[c]]/v.test('!abc')&&!/[[a]][[b]][[c]]\B/v.test('abc!')",
    );
    check(
        r"/\b[[K]][[s]][[A]]\b/iv.test('Kſa')&&!/\b[[K]][[s]][[A]]\b/v.test('Kſa')&&!/\b[[K]][[s]][[A]]\b/v.test('Ksa')&&/\b[[K]][[s]][[A]]\b/v.test('KsA')&&/\B[[a]][[a]][[^K]]\b/iv.exec('aaKaaſ!')[0]==='aaſ'&&!/\B[[a]][[a]][[^K]]\b/iv.test('aaK')",
    );
    check(
        r"/\b[[\p{Any}]][[a]][[a]]/iv.test('Kaa')&&!/\b[[\p{Any}]][[a]][[a]]/v.test('Kaa')&&/\b[[\p{Any}]][[a]][[a]]/iv.test('ſaa')&&!/\b[[\p{Any}]][[a]][[a]]/v.test('ſaa')&&/[[a]][[a]][[\p{Any}]]\b/iv.test('aaK!')&&!/[[a]][[a]][[\p{Any}]]\b/v.test('aaK!')&&/[[a]][[a]][[\p{Any}]]\b/iv.test('aaſ!')&&!/[[a]][[a]][[\p{Any}]]\b/v.test('aaſ!')",
    );
    check(
        r"/\b[\P{Ll}&&A][[x]][[s]]\b/v.test('Axs')&&!/\b[\P{Ll}&&A][[x]][[s]]\b/iv.test('Axs')&&/\b[[\p{ASCII}]][[x]][[a]]\b/v.test('Axa')&&!/\b[[\p{ASCII}]][[x]][[a]]\b/v.test('Kxa')",
    );
    check(
        r"let r=/\B[[\p{Any}]][[\u{1f601}]][[a]]\b/dvy;r.lastIndex=1;let m=r.exec('😀😁a!');m.index===0&&m[0]==='😀😁a'&&m.indices[0][1]===5&&r.lastIndex===5&&/\B[[😀]][[😁]][[😂]]\B/v.test('!😀😁😂!')",
    );
    check(
        r"let r=/\B[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]]\B/dvy;r.lastIndex=1;let m=r.exec('😀😁😂!');m.index===0&&m[0]==='😀😁😂'&&m.indices[0][1]===6&&r.lastIndex===6",
    );
    check(
        r"let r=/\b[[\uD800]][[\uD800]][[\uD800]]\B/dv,m=r.exec('A\uD800\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&!/\B[[\uD83D]][[\uDE00]][[a]]\b/v.test('😀a')&&!/\b[[a]][[\uD83D]][[\uDE00]]\B/v.test('a😀')",
    );
    check(
        r"let r=/\b[[a]][[\/]][[b]]\b/v;r.test('a/b')&&r.source==='\\b[[a]][[\\/]][[b]]\\b'&&/\b[[a]][[\[]][[b]]\b/v.test('a[b')&&/^\b[[a]][[$]][[b]]\b$/v.test('a$b')&&!/^\b[[a]][[$]][[b]]\b$/v.test('a$b\n')",
    );
    check(
        r"let r=/^\b[[a]][[b]][[c]]\b$/dvmg,m=r.exec('x\r\nabc\u2028de');m.index===3&&m[0]==='abc'&&m.indices[0][1]===6&&r.lastIndex===6&&r.exec('x\r\nabc\u2028de')===null&&r.lastIndex===0&&!/^\b[[a]][[b]][[c]]\b$/v.test('abc\n')&&/^\b[[a]][[b]][[c]]/v.test('abc!')&&/[[a]][[b]][[c]]\b$/v.test('xabc')",
    );
    check(
        r"let r=/(?<whole>(\B[[a]][[a]][[a]]\b))/dvg,a=[...'aaaa! aaaa!'.matchAll(r)];a.length===2&&a[0][0]==='aaa'&&a[0][1]==='aaa'&&a[0][2]==='aaa'&&a[0].index===1&&a[0].groups.whole==='aaa'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===4&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0",
    );
    check(
        r"'aaaa! aaaa!'.replace(/(?<whole>(\B[[a]][[a]][[a]]\b))/vg,'<$1:$<whole>>')==='a<aaa:aaa>! a<aaa:aaa>!'&&'aaaa!'.split(/(\B[[a]][[a]][[a]]\b)/v).join('|')==='a|aaa|!'&&'aaaa!'.match(/\B[[a]][[a]][[a]]\b/v)[0]==='aaa'",
    );
    check(
        r"let seen='',out='aaaa! aaaa!'.replace(/(?<whole>(\B[[a]][[a]][[a]]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aaa;7:aaa;'",
    );
    check(r"let r=/\B[[a]][[a]][[a]]\b/vg;r.lastIndex=4;'aaaa!'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/\B[[\p{Any}]][[\u{1f601}]][[a]]\b/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5",
    );
    check(
        r"let seen=0,r=/\B[[a]][[a]][[a]]\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaaa!');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\b[[K]][[s]][[A]]\b/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='\\b[[K]][[s]][[A]]\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſa')===null&&c.lastIndex===0&&r.exec('Kſa')[0]==='Kſa'",
    );
    check(
        r"let a=/\B[[a]][[a]][[a]]\b/vy,b=/\B[[a]][[a]][[a]]\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaaa!')}catch(e){x=e instanceof TypeError}try{b.exec('aaaa!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /\\b[a--a][[b]][c&&]\\b/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    check(r"/\b[[a]][[b]][[c]][[d]]\b/v.test('abcd')");
    check(
        r"let r=/\b[[a]][[b]][[c]][[d]]\b/v,m=r.exec('abcd');m!==null&&m.length===1&&m[0]==='abcd'&&m.index===0&&m.input==='abcd'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]][[b]][[c]][[d]]\\b'&&r.lastIndex===0&&!r.test('xbcd')&&!r.test('axcd')&&!r.test('abxd')&&!r.test('abcx')&&!r.test('abc')&&!r.test('xabcd')&&!r.test('abcdx')",
    );
    check(r"/\b[a][[b]][[c]]/v.test('abc')");
    check(
        r"let r=/\b[a][[b]][[c]]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[a][[b]][[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')&&r.exec('!abc').index===1&&r.exec('abcx').index===0&&r.exec('abcx')[0]==='abc'&&r.lastIndex===0",
    );
    check(r"/\b[[a]][b][[c]]/v.test('abc')");
    check(
        r"let r=/\b[[a]][b][[c]]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]][b][[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')&&r.exec('!abc').index===1&&r.exec('abcx').index===0&&r.exec('abcx')[0]==='abc'&&r.lastIndex===0",
    );
    check(r"/\b[[a]][[b]][c]/v.test('abc')");
    check(
        r"let r=/\b[[a]][[b]][c]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]][[b]][c]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')&&r.exec('!abc').index===1&&r.exec('abcx').index===0&&r.exec('abcx')[0]==='abc'&&r.lastIndex===0",
    );
    for source in [
        r"/\bx[[a]][[b]][[c]]/v.test('xabc')",
        r"/[[a]][[b]][[c]]x\b/v.test('abcx')",
        r"/\b^[[a]][[b]][[c]]/v.test('abc')",
        r"/[[a]][[b]][[c]]$\b/v.test('abc')",
        r"/\b\b[[a]][[b]][[c]]/v.test('abc')",
        r"/[[a]][[b]][[c]]\b\b/v.test('abc')",
        r"/\b\B[[a]][[b]][[c]]/v.test('abc')",
        r"/\b[[a]]\b[[b]][[c]]\b/v.test('abc')",
        r"/\b[[a]]^[[b]][[c]]/v.test('abc')",
        r"/(\b[[a]])[[b]][[c]]/v.test('abc')",
        r"/\b[[a]]([[b]])[[c]]/v.test('abc')",
        r"/\b[[a]][[b]]([[c]])/v.test('abc')",
        r"/\b[[a]]+[[b]][[c]]/v.test('abc')",
        r"/\b[[a]][[b]]?[[c]]/v.test('abc')",
        r"/\b[[a]][[b]][[c]]+/v.test('abc')",
        r"/(?:\b[[a]][[b]][[c]]\b)+/v.test('abc')",
        r"/\b[a--a][[b]][[\p{RGI_Emoji}]]\b/v.test('abc')",
        r"/\b[[a]][b--b][[\p{RGI_Emoji}]]\b/v.test('abc')",
        r"/\b[[\p{RGI_Emoji}]][[b]][c--c]\b/v.test('abc')",
        r"/\b[a--a][[b]][[\q{c}]]\b/v.test('abc')",
        r"/\b[[a]][b--b][[\q{c}]]\b/v.test('abc')",
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
        .eval(r"var kept=/(?<whole>(\B[[a]][[a]][[a]]\b))/dvg;kept.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"let m=kept.exec('aaaa!');m.index===1&&m.groups.whole==='aaa'&&m.indices[2][1]===4&&kept.lastIndex===4"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/\B[[a]][[a]][[c]]\b/vg,sticky=/\b[[a]][[a]][[a]]/vy,marker=0")
        .unwrap();
    // The first two checked classes succeed; the third fails before word gates.
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
fn checked_v_set_class_triple_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>(\B[[a]][[a]][[a]]\b))",
            JsString::from("aaaa!"),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\B[[a]][[a]][[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[a]][[a]][[a]]\b))",
            JsString::from("aaaa!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[[a]][[a]][[a]]\b))",
            JsString::from("aaaa!"),
            "dvy",
            1,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\B[[a]][[a]][[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[[K]][[s]][[A]]\b))",
            JsString::from("Kſa!"),
            "divg",
            0,
            r#"{"matches":["Kſa","Kſa","Kſa"],"index":0,"input":"Kſa!","groups":{"whole":"Kſa"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\b[[K]][[s]][[A]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[\P{Ll}&&A][[x]][[s]]\b))",
            JsString::from("Axs"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[[\p{Any}]][[\u{1f601}]][[a]]\b))",
            JsString::from("😀😁a!"),
            "dvy",
            1,
            r#"{"matches":["😀😁a","😀😁a","😀😁a"],"index":0,"input":"😀😁a!","groups":{"whole":"😀😁a"},"indices":[[0,5],[0,5],[0,5]],"indexGroups":{"whole":[0,5]},"lastIndex":5,"source":"(?<whole>(\\B[[\\p{Any}]][[\\u{1f601}]][[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]]\B))",
            JsString::from("!😀😁😂!"),
            "dvg",
            0,
            r#"{"matches":["😀😁😂","😀😁😂","😀😁😂"],"index":1,"input":"!😀😁😂!","groups":{"whole":"😀😁😂"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\B[[\\u{1f600}]][[\\u{1f601}]][[\\u{1f602}]]\\B))"}"#,
        ),
        (
            r"(?<whole>(\b[[\uD800]][[\uD800]][[\uD800]]\B))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b[[\\uD800]][[\\uD800]][[\\uD800]]\\B))"}"#,
        ),
        (
            r"(?<whole>(\B[[a]][[a]][[^K]]\b))",
            JsString::from("aaKaaſ!"),
            "divg",
            0,
            r#"{"matches":["aaſ","aaſ","aaſ"],"index":3,"input":"aaKaaſ!","groups":{"whole":"aaſ"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"(?<whole>(\\B[[a]][[a]][[^K]]\\b))"}"#,
        ),
        (
            r"^(?<whole>(\b[[a]][[b]][[c]]\b))$",
            JsString::from("x\r\nabc\u{2028}de"),
            "dvmg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":3,"input":"x\r\nabc de","groups":{"whole":"abc"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"^(?<whole>(\\b[[a]][[b]][[c]]\\b))$"}"#,
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
fn literal_checked_v_set_pair_word_state_context_and_consumers() {
    check(
        r"/\bc[a--b][[c]]\b/v.test('cac')&&!/\bc[a--b][[c]]\b/v.test('cbc')&&!/\bc[a--b][[c]]\b/v.test('ca')&&!/\bc[a--a][[b]]\b/v.test('cab')&&!/\bc[[a]][b--b]\b/v.test('cab')",
    );
    check(r#"let marker=0;eval("marker=1; /\\bc[a--b][[c]]\\b/v.test('cac')")&&marker===1"#);
    check(
        r"let a=/a[[a]][[a]]\b/dvg,b=/\Ba[[a]][[a]]/dvg,x=a.exec('aaaa!'),y=b.exec('aaaa!');x.index===1&&x[0]==='aaa'&&x.indices[0][1]===4&&a.lastIndex===4&&y.index===1&&y[0]==='aaa'&&y.indices[0][1]===4&&b.lastIndex===4",
    );
    check(
        r"let g=/\Ba[[a]][[a]]\b/vg,y=/\Ba[[a]][[a]]\b/vy,a=g.exec('aaaa!');a.index===1&&a[0]==='aaa'&&g.lastIndex===4&&g.exec('aaaa!')===null&&g.lastIndex===0&&y.exec('aaaa!')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaaa!')[0]==='aaa')&&y.lastIndex===4",
    );
    check(
        r"/\bc[[a]][[b]]/v.test('cab')&&/\Bc[[a]][[b]]/v.test('xcab')&&/c[[a]][[b]]\b/v.test('cab!')&&/c[[a]][[b]]\B/v.test('cabx')&&/\bc[[a]][[b]]\b/v.test('!cab!')&&/\bc[[a]][[b]]\B/v.test('!cabx')&&/\Bc[[a]][[b]]\b/v.test('xcab!')&&/\Bc[[a]][[b]]\B/v.test('xcabx')",
    );
    check(
        r"!/\bc[[a]][[b]]/v.test('xcab')&&!/c[[a]][[b]]\b/v.test('cabx')&&!/\Bc[[a]][[b]]/v.test('!cab')&&!/c[[a]][[b]]\B/v.test('cab!')",
    );
    check(
        r"/\bk[[s]][[A]]\b/iv.test('Kſa')&&!/\bk[[s]][[A]]\b/v.test('Kſa')&&!/\bk[[s]][[A]]\b/v.test('Ksa')&&/\bk[[s]][[A]]\b/v.test('ksA')&&/\Ba[[a]][[^K]]\b/iv.exec('aaKaaſ!')[0]==='aaſ'&&!/\Ba[[a]][[^K]]\b/iv.test('aaK')",
    );
    check(
        r"/\b\u{212a}[[a]][[a]]/iv.test('Kaa')&&!/\b\u{212a}[[a]][[a]]/v.test('Kaa')&&/\b\u{17f}[[a]][[a]]/iv.test('ſaa')&&!/\b\u{17f}[[a]][[a]]/v.test('ſaa')&&/a[[a]][[\p{Any}]]\b/iv.test('aaK!')&&!/a[[a]][[\p{Any}]]\b/v.test('aaK!')&&/a[[a]][[\p{Any}]]\b/iv.test('aaſ!')&&!/a[[a]][[\p{Any}]]\b/v.test('aaſ!')",
    );
    check(
        r"/\bx[\P{Ll}&&A][[s]]\b/v.test('xAs')&&!/\bx[\P{Ll}&&A][[s]]\b/iv.test('xAs')&&/\bx[[\p{ASCII}]][[a]]\b/v.test('xAa')&&!/\bx[[\p{ASCII}]][[a]]\b/v.test('xKa')",
    );
    check(
        r"let r=/\B\u{1f600}[[\p{Any}]][[a]]\b/dvy;r.lastIndex=1;let m=r.exec('😀😁a!');m.index===0&&m[0]==='😀😁a'&&m.indices[0][1]===5&&r.lastIndex===5&&/\B😀[[😁]][[😂]]\B/v.test('!😀😁😂!')&&/\B\uD83D\uDE00[[\u{1f601}]][[\u{1f602}]]\B/v.test('!😀😁😂!')",
    );
    check(
        r"let r=/\B\u{1f600}[[\u{1f601}]][[\u{1f602}]]\B/dvy;r.lastIndex=1;let m=r.exec('😀😁😂!');m.index===0&&m[0]==='😀😁😂'&&m.indices[0][1]===6&&r.lastIndex===6",
    );
    check(
        r"let r=/\b\uD800[[\uD800]][[\uD800]]\B/dv,m=r.exec('A\uD800\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&!/\B\uD83D[[\uDE00]][[a]]\b/v.test('😀a')&&!/\ba[[\uD83D]][[\uDE00]]\B/v.test('a😀')",
    );
    check(
        r"let r=/\B\/[[a]][[b]]\b/v;r.test('!/ab!')&&r.source==='\\B\\/[[a]][[b]]\\b'&&/\b\[[[a]][[b]]\b/v.test('A[ab!')&&/^\B\$[[a]][[b]]\b$/v.test('$ab')&&!/^\B\$[[a]][[b]]\b$/v.test('$ab\n')",
    );
    check(
        r"let r=/^\bc[[a]][[b]]\b$/dvmg,m=r.exec('x\r\ncab\u2028cd');m.index===3&&m[0]==='cab'&&m.indices[0][1]===6&&r.lastIndex===6&&r.exec('x\r\ncab\u2028cd')===null&&r.lastIndex===0&&!/^\bc[[a]][[b]]\b$/v.test('cab\n')&&/^\bc[[a]][[b]]/v.test('cab!')&&/c[[a]][[b]]\b$/v.test('xcab')",
    );
    check(
        r"let r=/(?<whole>(\Ba[[a]][[a]]\b))/dvg,a=[...'aaaa! aaaa!'.matchAll(r)];a.length===2&&a[0][0]==='aaa'&&a[0][1]==='aaa'&&a[0][2]==='aaa'&&a[0].index===1&&a[0].groups.whole==='aaa'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===4&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0",
    );
    check(
        r"'aaaa! aaaa!'.replace(/(?<whole>(\Ba[[a]][[a]]\b))/vg,'<$1:$<whole>>')==='a<aaa:aaa>! a<aaa:aaa>!'&&'aaaa!'.split(/(\Ba[[a]][[a]]\b)/v).join('|')==='a|aaa|!'&&'aaaa!'.match(/\Ba[[a]][[a]]\b/v)[0]==='aaa'",
    );
    check(
        r"let seen='',out='aaaa! aaaa!'.replace(/(?<whole>(\Ba[[a]][[a]]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aaa;7:aaa;'",
    );
    check(r"let r=/\Ba[[a]][[a]]\b/vg;r.lastIndex=4;'aaaa!'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/\B\u{1f600}[[\p{Any}]][[a]]\b/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5",
    );
    check(
        r"let seen=0,r=/\Ba[[a]][[a]]\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaaa!');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\bk[[s]][[A]]\b/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='\\bk[[s]][[A]]\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſa')===null&&c.lastIndex===0&&r.exec('Kſa')[0]==='Kſa'",
    );
    check(
        r"let a=/\Ba[[a]][[a]]\b/vy,b=/\Ba[[a]][[a]]\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaaa!')}catch(e){x=e instanceof TypeError}try{b.exec('aaaa!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /\\bc[a--a][b&&]\\b/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    check(r"/\b[[a]][[b]]c/v.test('abc')");
    check(
        r"let r=/\b[[a]][[b]]c/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]][[b]]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')&&r.test('abcx')",
    );
    check(r"/[[a]][[b]]c\b/v.test('abc')");
    check(
        r"let r=/[[a]][[b]]c\b/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][[b]]c\\b'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('abcx')&&r.test('xabc')",
    );
    check(r"/\b[[a]]b[[c]]/v.test('abc')");
    check(
        r"let r=/\b[[a]]b[[c]]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]]b[[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')&&r.test('abcx')",
    );
    check(r"/[[a]]b[[c]]\b/v.test('abc')");
    check(
        r"let r=/[[a]]b[[c]]\b/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]]b[[c]]\\b'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('abcx')&&r.test('xabc')",
    );
    check(r"/\bc[a][b]/v.test('cab')");
    check(
        r#"let r=/\bc[a][b]/v,m=r.exec('cab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\bc[a][b]'&&r.lastIndex===0&&!r.test('xab')&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&!r.test('xcab')&&r.test('cabx')"#,
    );
    check(r"/\bc[[a]][b]/v.test('cab')");
    check(
        r#"let r=/\bc[[a]][b]/v,m=r.exec('cab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\bc[[a]][b]'&&r.lastIndex===0&&!r.test('xab')&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&r.test('cabx')&&!r.test('xcab')&&r.exec('!cab').index===1"#,
    );
    check(r"/\bc[a][[b]]/v.test('cab')");
    check(
        r#"let r=/\bc[a][[b]]/v,m=r.exec('cab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\bc[a][[b]]'&&r.lastIndex===0&&!r.test('xab')&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&r.test('cabx')&&!r.test('xcab')&&r.exec('!cab').index===1"#,
    );
    for source in [
        r"/\bca[[a]][[b]]/v.test('caab')",
        r"/\bc[[a]][[b]]d/v.test('cabd')",
        r"/\bc[[a]][[b]][[c]]/v.test('cabc')",
        r"/\bc[[a]]\b[[b]]\b/v.test('cab')",
        r"/\b^c[[a]][[b]]/v.test('cab')",
        r"/c[[a]][[b]]$\b/v.test('cab')",
        r"/\b\bc[[a]][[b]]/v.test('cab')",
        r"/c[[a]][[b]]\b\b/v.test('cab')",
        r"/(\bc)[[a]][[b]]/v.test('cab')",
        r"/\bc([[a]])[[b]]/v.test('cab')",
        r"/\bc[[a]]([[b]])/v.test('cab')",
        r"/\bc+[[a]][[b]]/v.test('cab')",
        r"/\bc[[a]]+[[b]]/v.test('cab')",
        r"/\bc[[a]][[b]]?/v.test('cab')",
        r"/(?:\bc[[a]][[b]]\b)+/v.test('cab')",
        r"/\bc[a--a][[\q{b}]]\b/v.test('cab')",
        r"/\bc[[\q{a}]][b--b]\b/v.test('cab')",
        r"/\bc[a--a][[\p{RGI_Emoji}]]\b/v.test('cab')",
        r"/\bc[[\p{RGI_Emoji}]][b--b]\b/v.test('cab')",
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
        .eval(r"var kept=/(?<whole>(\Ba[[a]][[a]]\b))/dvg;kept.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"let m=kept.exec('aaaa!');m.index===1&&m.groups.whole==='aaa'&&m.indices[2][1]===4&&kept.lastIndex===4"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/\Ba[[a]][[c]]\b/vg,sticky=/\ba[[a]][[a]]/vy,marker=0")
        .unwrap();
    // The literal and first class succeed; the second fails before word gates.
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
fn literal_checked_v_set_pair_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>(\Ba[[a]][[a]]\b))",
            JsString::from("aaaa!"),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\Ba[[a]][[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\Ba[[a]][[a]]\b))",
            JsString::from("aaaa!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\Ba[[a]][[a]]\b))",
            JsString::from("aaaa!"),
            "dvy",
            1,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\Ba[[a]][[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\bk[[s]][[A]]\b))",
            JsString::from("Kſa!"),
            "divg",
            0,
            r#"{"matches":["Kſa","Kſa","Kſa"],"index":0,"input":"Kſa!","groups":{"whole":"Kſa"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\bk[[s]][[A]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\bx[\P{Ll}&&A][[s]]\b))",
            JsString::from("xAs!"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B\u{1f600}[[\p{Any}]][[a]]\b))",
            JsString::from("😀😁a!"),
            "dvy",
            1,
            r#"{"matches":["😀😁a","😀😁a","😀😁a"],"index":0,"input":"😀😁a!","groups":{"whole":"😀😁a"},"indices":[[0,5],[0,5],[0,5]],"indexGroups":{"whole":[0,5]},"lastIndex":5,"source":"(?<whole>(\\B\\u{1f600}[[\\p{Any}]][[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B\u{1f600}[[\u{1f601}]][[\u{1f602}]]\B))",
            JsString::from("!😀😁😂!"),
            "dvg",
            0,
            r#"{"matches":["😀😁😂","😀😁😂","😀😁😂"],"index":1,"input":"!😀😁😂!","groups":{"whole":"😀😁😂"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\B\\u{1f600}[[\\u{1f601}]][[\\u{1f602}]]\\B))"}"#,
        ),
        (
            r"(?<whole>(\b\uD800[[\uD800]][[\uD800]]\B))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b\\uD800[[\\uD800]][[\\uD800]]\\B))"}"#,
        ),
        (
            r"(?<whole>(\ba[[a]][[\p{Any}]]\b))",
            JsString::from("aaK!"),
            "divg",
            0,
            r#"{"matches":["aaK","aaK","aaK"],"index":0,"input":"aaK!","groups":{"whole":"aaK"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\ba[[a]][[\\p{Any}]]\\b))"}"#,
        ),
        (
            r"^(?<whole>(\bc[[a]][[b]]\b))$",
            JsString::from("x\r\ncab\u{2028}cd"),
            "dvmg",
            0,
            r#"{"matches":["cab","cab","cab"],"index":3,"input":"x\r\ncab cd","groups":{"whole":"cab"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"^(?<whole>(\\bc[[a]][[b]]\\b))$"}"#,
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
fn checked_v_set_pair_literal_word_state_context_and_consumers() {
    check(
        r"/\b[a--b][[c]]d\b/v.test('acd')&&!/\b[a--b][[c]]d\b/v.test('bcd')&&!/\b[a--b][[c]]d\b/v.test('axd')&&!/\b[a--b][[c]]d\b/v.test('acx')&&!/\b[a--b][[c]]d\b/v.test('ac')&&!/\b[a--a][[c]]d\b/v.test('acd')&&!/\b[[a]][c--c]d\b/v.test('acd')",
    );
    check(r#"let marker=0;eval("marker=1; /\\b[a--b][[c]]d\\b/v.test('acd')")&&marker===1"#);
    check(
        r"let a=/[[a]][[a]]a\b/dvg,b=/\B[[a]][[a]]a/dvg,x=a.exec('aaaa!'),y=b.exec('aaaa!');x.index===1&&x[0]==='aaa'&&x.indices[0][1]===4&&a.lastIndex===4&&y.index===1&&y[0]==='aaa'&&y.indices[0][1]===4&&b.lastIndex===4",
    );
    check(
        r"let g=/\B[[a]][[a]]a\b/vg,y=/\B[[a]][[a]]a\b/vy,a=g.exec('aaaa!');a.index===1&&a[0]==='aaa'&&g.lastIndex===4&&g.exec('aaaa!')===null&&g.lastIndex===0&&y.exec('aaaa!')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaaa!')[0]==='aaa')&&y.lastIndex===4",
    );
    check(
        r"/\b[[a]][[b]]c/v.test('abc')&&/\B[[a]][[b]]c/v.test('xabc')&&/[[a]][[b]]c\b/v.test('abc!')&&/[[a]][[b]]c\B/v.test('abcx')&&/\b[[a]][[b]]c\b/v.test('!abc!')&&/\b[[a]][[b]]c\B/v.test('!abcx')&&/\B[[a]][[b]]c\b/v.test('xabc!')&&/\B[[a]][[b]]c\B/v.test('xabcx')",
    );
    check(
        r"!/\b[[a]][[b]]c/v.test('xabc')&&!/[[a]][[b]]c\b/v.test('abcx')&&!/\B[[a]][[b]]c/v.test('!abc')&&!/[[a]][[b]]c\B/v.test('abc!')",
    );
    check(
        r"/\b[[K]][[s]]a\b/iv.test('KſA')&&!/\b[[K]][[s]]a\b/v.test('KſA')&&!/\b[[K]][[s]]a\b/v.test('KsA')&&/\b[[K]][[s]]a\b/v.test('Ksa')&&/\B[[a]][[^K]]s\b/iv.exec('aKsaſſ!')[0]==='aſſ'&&!/\B[[a]][[^K]]s\b/iv.test('aKs')",
    );
    check(
        r"/\b[[\p{Any}]][[a]]a/iv.test('Kaa')&&!/\b[[\p{Any}]][[a]]a/v.test('Kaa')&&/\b[[\p{Any}]][[a]]a/iv.test('ſaa')&&!/\b[[\p{Any}]][[a]]a/v.test('ſaa')&&/[[a]][[a]]\u{212a}\b/iv.test('aaK!')&&!/[[a]][[a]]\u{212a}\b/v.test('aaK!')&&/[[a]][[a]]\u{17f}\b/iv.test('aaſ!')&&!/[[a]][[a]]\u{17f}\b/v.test('aaſ!')",
    );
    check(
        r"/\b[\P{Ll}&&A][[s]]x\b/v.test('Asx')&&!/\b[\P{Ll}&&A][[s]]x\b/iv.test('Asx')&&/\b[[\p{ASCII}]][[a]]x\b/v.test('Aax')&&!/\b[[\p{ASCII}]][[a]]x\b/v.test('Kax')",
    );
    check(
        r"let r=/\B[[\p{Any}]][[\u{1f601}]]a\b/dvy;r.lastIndex=1;let m=r.exec('😀😁a!');m.index===0&&m[0]==='😀😁a'&&m.indices[0][1]===5&&r.lastIndex===5&&/\B[[😀]][[😁]]😂\B/v.test('!😀😁😂!')&&/\B[[\u{1f600}]][[\u{1f601}]]\uD83D\uDE02\B/v.test('!😀😁😂!')",
    );
    check(
        r"let r=/\B[[\u{1f600}]][[\u{1f601}]]\u{1f602}\B/dvy;r.lastIndex=1;let m=r.exec('😀😁😂!');m.index===0&&m[0]==='😀😁😂'&&m.indices[0][1]===6&&r.lastIndex===6",
    );
    check(
        r"let r=/\b[[\uD800]][[\uD800]]\uD800\B/dv,m=r.exec('A\uD800\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&!/\B[[\uD83D]][[\uDE00]]a\b/v.test('😀a')&&!/\b[[a]][[\uD83D]]\uDE00\B/v.test('a😀')",
    );
    check(
        r"let r=/\b[[a]][[b]]\/\B/v;r.test('ab/!')&&r.source==='\\b[[a]][[b]]\\/\\B'&&/\b[[a]][[b]]\[\B/v.test('ab[!')&&/^\b[[a]][[b]]\$\B$/v.test('ab$')&&!/^\b[[a]][[b]]\$\B$/v.test('ab$\n')",
    );
    check(
        r"let r=/^\b[[a]][[b]]c\b$/dvmg,m=r.exec('x\r\nabc\u2028de');m.index===3&&m[0]==='abc'&&m.indices[0][1]===6&&r.lastIndex===6&&r.exec('x\r\nabc\u2028de')===null&&r.lastIndex===0&&!/^\b[[a]][[b]]c\b$/v.test('abc\n')&&/^\b[[a]][[b]]c/v.test('abc!')&&/[[a]][[b]]c\b$/v.test('xabc')",
    );
    check(
        r"let r=/(?<whole>(\B[[a]][[a]]a\b))/dvg,a=[...'aaaa! aaaa!'.matchAll(r)];a.length===2&&a[0][0]==='aaa'&&a[0][1]==='aaa'&&a[0][2]==='aaa'&&a[0].index===1&&a[0].groups.whole==='aaa'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===4&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0",
    );
    check(
        r"'aaaa! aaaa!'.replace(/(?<whole>(\B[[a]][[a]]a\b))/vg,'<$1:$<whole>>')==='a<aaa:aaa>! a<aaa:aaa>!'&&'aaaa!'.split(/(\B[[a]][[a]]a\b)/v).join('|')==='a|aaa|!'&&'aaaa!'.match(/\B[[a]][[a]]a\b/v)[0]==='aaa'",
    );
    check(
        r"let seen='',out='aaaa! aaaa!'.replace(/(?<whole>(\B[[a]][[a]]a\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aaa;7:aaa;'",
    );
    check(r"let r=/\B[[a]][[a]]a\b/vg;r.lastIndex=4;'aaaa!'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/\B[[\p{Any}]][[\u{1f601}]]a\b/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5",
    );
    check(
        r"let seen=0,r=/\B[[a]][[a]]a\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaaa!');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\b[[K]][[s]]a\b/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='\\b[[K]][[s]]a\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſA')===null&&c.lastIndex===0&&r.exec('KſA')[0]==='KſA'",
    );
    check(
        r"let a=/\B[[a]][[a]]a\b/vy,b=/\B[[a]][[a]]a\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaaa!')}catch(e){x=e instanceof TypeError}try{b.exec('aaaa!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /\\b[a--a][b&&]c\\b/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    check(r"/\b[[a]]b[[c]]/v.test('abc')");
    check(
        r"let r=/\b[[a]]b[[c]]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]]b[[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')&&r.test('abcx')",
    );
    check(r"/[[a]]b[[c]]\b/v.test('abc')");
    check(
        r"let r=/[[a]]b[[c]]\b/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]]b[[c]]\\b'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('abcx')&&r.test('xabc')",
    );
    check(r"/\b[a][b]c/v.test('abc')");
    check(
        r"let r=/\b[a][b]c/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[a][b]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')&&r.test('abcx')",
    );
    check(r"/\b[[a]][b]c/v.test('abc')");
    check(
        r#"let r=/\b[[a]][b]c/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]][b]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&!r.test('xabc')&&r.exec('!abc').index===1"#,
    );
    check(r"/\b[a][[b]]c/v.test('abc')");
    check(
        r#"let r=/\b[a][[b]]c/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[a][[b]]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&!r.test('xabc')&&r.exec('!abc').index===1"#,
    );
    for source in [
        r"/\b[[a]][[b]]cd/v.test('abcd')",
        r"/\b[[a]][[b]][[c]]d/v.test('abcd')",
        r"/\b[[a]][[b]]c[[d]]/v.test('abcd')",
        r"/[[a]][[b]]\bc/v.test('abc')",
        r"/\b[[a]]\b[[b]]c\b/v.test('abc')",
        r"/\b^[[a]][[b]]c/v.test('abc')",
        r"/[[a]][[b]]c$\b/v.test('abc')",
        r"/\b\b[[a]][[b]]c/v.test('abc')",
        r"/[[a]][[b]]c\b\b/v.test('abc')",
        r"/(\b[[a]])[[b]]c/v.test('abc')",
        r"/\b[[a]]([[b]])c/v.test('abc')",
        r"/\b[[a]][[b]](c)/v.test('abc')",
        r"/\b[[a]]+[[b]]c/v.test('abc')",
        r"/\b[[a]][[b]]?c/v.test('abc')",
        r"/\b[[a]][[b]]c+/v.test('abc')",
        r"/(?:\b[[a]][[b]]c\b)+/v.test('abc')",
        r"/\b[a--a][[\q{b}]]c\b/v.test('abc')",
        r"/\b[[\q{a}]][b--b]c\b/v.test('abc')",
        r"/\b[a--a][[\p{RGI_Emoji}]]c\b/v.test('abc')",
        r"/\b[[\p{RGI_Emoji}]][b--b]c\b/v.test('abc')",
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
        .eval(r"var kept=/(?<whole>(\B[[a]][[a]]a\b))/dvg;kept.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"let m=kept.exec('aaaa!');m.index===1&&m.groups.whole==='aaa'&&m.indices[2][1]===4&&kept.lastIndex===4"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/\B[[a]][[a]]c\b/vg,sticky=/\b[[a]][[a]]a/vy,marker=0")
        .unwrap();
    // Both classes succeed; the suffix literal fails before word gates.
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
fn checked_v_set_pair_literal_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>(\B[[a]][[a]]a\b))",
            JsString::from("aaaa!"),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\B[[a]][[a]]a\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[a]][[a]]a\b))",
            JsString::from("aaaa!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[[a]][[a]]a\b))",
            JsString::from("aaaa!"),
            "dvy",
            1,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\B[[a]][[a]]a\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[[K]][[s]]a\b))",
            JsString::from("KſA!"),
            "divg",
            0,
            r#"{"matches":["KſA","KſA","KſA"],"index":0,"input":"KſA!","groups":{"whole":"KſA"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\b[[K]][[s]]a\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[\P{Ll}&&A][[s]]x\b))",
            JsString::from("Asx!"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[[\p{Any}]][[\u{1f601}]]a\b))",
            JsString::from("😀😁a!"),
            "dvy",
            1,
            r#"{"matches":["😀😁a","😀😁a","😀😁a"],"index":0,"input":"😀😁a!","groups":{"whole":"😀😁a"},"indices":[[0,5],[0,5],[0,5]],"indexGroups":{"whole":[0,5]},"lastIndex":5,"source":"(?<whole>(\\B[[\\p{Any}]][[\\u{1f601}]]a\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[\u{1f600}]][[\u{1f601}]]\u{1f602}\B))",
            JsString::from("!😀😁😂!"),
            "dvg",
            0,
            r#"{"matches":["😀😁😂","😀😁😂","😀😁😂"],"index":1,"input":"!😀😁😂!","groups":{"whole":"😀😁😂"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\B[[\\u{1f600}]][[\\u{1f601}]]\\u{1f602}\\B))"}"#,
        ),
        (
            r"(?<whole>(\b[[\uD800]][[\uD800]]\uD800\B))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b[[\\uD800]][[\\uD800]]\\uD800\\B))"}"#,
        ),
        (
            r"(?<whole>(\b[[\p{Any}]][[a]]a\b))",
            JsString::from("Kaa!"),
            "divg",
            0,
            r#"{"matches":["Kaa","Kaa","Kaa"],"index":0,"input":"Kaa!","groups":{"whole":"Kaa"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\b[[\\p{Any}]][[a]]a\\b))"}"#,
        ),
        (
            r"^(?<whole>(\b[[a]][[b]]c\b))$",
            JsString::from("x\r\nabc\u{2028}de"),
            "dvmg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":3,"input":"x\r\nabc de","groups":{"whole":"abc"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"^(?<whole>(\\b[[a]][[b]]c\\b))$"}"#,
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
fn checked_v_set_literal_set_word_state_context_and_consumers() {
    check(
        r"/\b[a--b]c[[d]]\b/v.test('acd')&&!/\b[a--b]c[[d]]\b/v.test('bcd')&&!/\b[a--b]c[[d]]\b/v.test('axd')&&!/\b[a--b]c[[d]]\b/v.test('acx')&&!/\b[a--b]c[[d]]\b/v.test('ac')&&!/\b[a--a]c[[d]]\b/v.test('acd')&&!/\b[[a]]c[d--d]\b/v.test('acd')",
    );
    check(r#"let marker=0;eval("marker=1; /\\b[a--b]c[[d]]\\b/v.test('acd')")&&marker===1"#);
    check(
        r"let a=/[[a]]a[[a]]\b/dvg,b=/\B[[a]]a[[a]]/dvg,x=a.exec('aaaa!'),y=b.exec('aaaa!');x.index===1&&x[0]==='aaa'&&x.indices[0][1]===4&&a.lastIndex===4&&y.index===1&&y[0]==='aaa'&&y.indices[0][1]===4&&b.lastIndex===4",
    );
    check(
        r"let g=/\B[[a]]a[[a]]\b/vg,y=/\B[[a]]a[[a]]\b/vy,a=g.exec('aaaa!');a.index===1&&a[0]==='aaa'&&g.lastIndex===4&&g.exec('aaaa!')===null&&g.lastIndex===0&&y.exec('aaaa!')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaaa!')[0]==='aaa')&&y.lastIndex===4",
    );
    check(
        r"/\b[[a]]b[[c]]/v.test('abc')&&/\B[[a]]b[[c]]/v.test('xabc')&&/[[a]]b[[c]]\b/v.test('abc!')&&/[[a]]b[[c]]\B/v.test('abcx')&&/\b[[a]]b[[c]]\b/v.test('!abc!')&&/\b[[a]]b[[c]]\B/v.test('!abcx')&&/\B[[a]]b[[c]]\b/v.test('xabc!')&&/\B[[a]]b[[c]]\B/v.test('xabcx')",
    );
    check(
        r"!/\b[[a]]b[[c]]/v.test('xabc')&&!/[[a]]b[[c]]\b/v.test('abcx')&&!/\B[[a]]b[[c]]/v.test('!abc')&&!/[[a]]b[[c]]\B/v.test('abc!')",
    );
    check(
        r"/\b[[K]]s[[a]]\b/iv.test('KſA')&&!/\b[[K]]s[[a]]\b/v.test('Ksa')&&!/\b[[K]]s[[a]]\b/v.test('Kſa')&&!/\b[[K]]s[[a]]\b/v.test('KsA')&&/\b[[K]]s[[a]]\b/v.test('Ksa')&&/\B[[a]]s[[^K]]\b/iv.exec('xasKasſ!')[0]==='asſ'&&!/\B[[a]]s[[^K]]\b/iv.test('xasK')",
    );
    check(
        r"/\b[[\p{Any}]]a[[a]]/iv.test('Kaa')&&!/\b[[\p{Any}]]a[[a]]/v.test('Kaa')&&/\b[[\p{Any}]]a[[a]]/iv.test('ſaa')&&!/\b[[\p{Any}]]a[[a]]/v.test('ſaa')&&/[[a]]a[[\p{Any}]]\b/iv.test('aaK!')&&!/[[a]]a[[\p{Any}]]\b/v.test('aaK!')&&/[[a]]a[[\p{Any}]]\b/iv.test('aaſ!')&&!/[[a]]a[[\p{Any}]]\b/v.test('aaſ!')",
    );
    check(
        r"/\b[\P{Ll}&&A]s[[x]]\b/v.test('Asx')&&!/\b[\P{Ll}&&A]s[[x]]\b/iv.test('Asx')&&/\b[[\p{ASCII}]]a[[x]]\b/v.test('Aax')&&!/\b[[\p{ASCII}]]a[[x]]\b/v.test('Kax')",
    );
    check(
        r"let r=/\B[[\p{Any}]]\u{1f601}[[a]]\b/dvy;r.lastIndex=1;let m=r.exec('😀😁a!');m.index===0&&m[0]==='😀😁a'&&m.indices[0][1]===5&&r.lastIndex===5&&/\B[[😀]]😁[[😂]]\B/v.test('!😀😁😂!')&&/\B[[\u{1f600}]]\uD83D\uDE01[[\u{1f602}]]\B/v.test('!😀😁😂!')",
    );
    check(
        r"let r=/\B[[\u{1f600}]]\u{1f601}[[\u{1f602}]]\B/dvy;r.lastIndex=1;let m=r.exec('😀😁😂!');m.index===0&&m[0]==='😀😁😂'&&m.indices[0][1]===6&&r.lastIndex===6",
    );
    check(
        r"let r=/\b[[\uD800]]\uD800[[\uD800]]\B/dv,m=r.exec('A\uD800\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&!/\B[[\uD83D]]\uDE00[[a]]\b/v.test('😀a')&&!/\b[[a]]\uD83D[[\uDE00]]\B/v.test('a😀')",
    );
    check(
        r"let r=/\b[[a]]\/[[b]]\b/v;r.test('a/b!')&&r.source==='\\b[[a]]\\/[[b]]\\b'&&/\b[[a]]\[[[b]]\b/v.test('a[b!')&&/^\b[[a]]\$[[b]]\b$/v.test('a$b')&&!/^\b[[a]]\$[[b]]\b$/v.test('a$b\n')",
    );
    check(
        r"let r=/^\b[[a]]b[[c]]\b$/dvmg,m=r.exec('x\r\nabc\u2028de');m.index===3&&m[0]==='abc'&&m.indices[0][1]===6&&r.lastIndex===6&&r.exec('x\r\nabc\u2028de')===null&&r.lastIndex===0&&!/^\b[[a]]b[[c]]\b$/v.test('abc\n')&&/^\b[[a]]b[[c]]/v.test('abc!')&&/[[a]]b[[c]]\b$/v.test('xabc')",
    );
    check(
        r"let r=/(?<whole>(\B[[a]]a[[a]]\b))/dvg,a=[...'aaaa! aaaa!'.matchAll(r)];a.length===2&&a[0][0]==='aaa'&&a[0][1]==='aaa'&&a[0][2]==='aaa'&&a[0].index===1&&a[0].groups.whole==='aaa'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===4&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0",
    );
    check(
        r"'aaaa! aaaa!'.replace(/(?<whole>(\B[[a]]a[[a]]\b))/vg,'<$1:$<whole>>')==='a<aaa:aaa>! a<aaa:aaa>!'&&'aaaa!'.split(/(\B[[a]]a[[a]]\b)/v).join('|')==='a|aaa|!'&&'aaaa!'.match(/\B[[a]]a[[a]]\b/v)[0]==='aaa'",
    );
    check(
        r"let seen='',out='aaaa! aaaa!'.replace(/(?<whole>(\B[[a]]a[[a]]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aaa;7:aaa;'",
    );
    check(r"let r=/\B[[a]]a[[a]]\b/vg;r.lastIndex=4;'aaaa!'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/\B[[\p{Any}]]\u{1f601}[[a]]\b/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5",
    );
    check(
        r"let seen=0,r=/\B[[a]]a[[a]]\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaaa!');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\b[[K]]s[[a]]\b/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='\\b[[K]]s[[a]]\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſA')===null&&c.lastIndex===0&&r.exec('KſA')[0]==='KſA'",
    );
    check(
        r"let a=/\B[[a]]a[[a]]\b/vy,b=/\B[[a]]a[[a]]\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaaa!')}catch(e){x=e instanceof TypeError}try{b.exec('aaaa!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /\\b[a--a]b[c&&]\\b/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    check(r"/\b[a]b[c]/v.test('abc')");
    check(
        r#"let r=/\b[a]b[c]/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[a]b[c]'&&r.lastIndex===0&&r.exec('xbc')===null&&r.exec('axc')===null&&r.exec('abd')===null&&r.exec('ab')===null&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc')===null&&r.exec('!abc').index===1"#,
    );
    check(r"/\b[[a]]b[c]/v.test('abc')");
    check(
        r#"let r=/\b[[a]]b[c]/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]]b[c]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&!r.test('xabc')&&r.exec('!abc').index===1"#,
    );
    check(r"/\b[a]b[[c]]/v.test('abc')");
    check(
        r#"let r=/\b[a]b[[c]]/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[a]b[[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&!r.test('xabc')&&r.exec('!abc').index===1"#,
    );
    for source in [
        r"/\b[[a]]bc[[d]]/v.test('abcd')",
        r"/\b[[a]]b[[c]][[d]]/v.test('abcd')",
        r"/\b[[a]][[b]]c[[d]]/v.test('abcd')",
        r"/\b[[a]]b[[c]]d/v.test('abcd')",
        r"/[[a]]\bb[[c]]/v.test('abc')",
        r"/\b[[a]]b\b[[c]]\b/v.test('abc')",
        r"/\b^[[a]]b[[c]]/v.test('abc')",
        r"/[[a]]b[[c]]$\b/v.test('abc')",
        r"/\b\b[[a]]b[[c]]/v.test('abc')",
        r"/[[a]]b[[c]]\b\b/v.test('abc')",
        r"/(\b[[a]])b[[c]]/v.test('abc')",
        r"/\b[[a]](b)[[c]]/v.test('abc')",
        r"/\b[[a]]b([[c]])/v.test('abc')",
        r"/\b[[a]]+b[[c]]/v.test('abc')",
        r"/\b[[a]]b?[[c]]/v.test('abc')",
        r"/\b[[a]]b[[c]]+/v.test('abc')",
        r"/(?:\b[[a]]b[[c]]\b)+/v.test('abc')",
        r"/\b[a--a]b[[\q{c}]]\b/v.test('abc')",
        r"/\b[[\q{a}]]b[c--c]\b/v.test('abc')",
        r"/\b[a--a]b[[\p{RGI_Emoji}]]\b/v.test('abc')",
        r"/\b[[\p{RGI_Emoji}]]b[c--c]\b/v.test('abc')",
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
        .eval(r"var kept=/(?<whole>(\B[[a]]a[[a]]\b))/dvg;kept.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"let m=kept.exec('aaaa!');m.index===1&&m.groups.whole==='aaa'&&m.indices[2][1]===4&&kept.lastIndex===4"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/\B[[a]]a[[c]]\b/vg,sticky=/\b[[a]]a[[a]]/vy,marker=0")
        .unwrap();
    // The first class and middle literal succeed; the final class fails.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval(r"sticky.test(text)&&sticky.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval(r"try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r"marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn checked_v_set_literal_set_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>(\B[[a]]a[[a]]\b))",
            JsString::from("aaaa!"),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\B[[a]]a[[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[a]]a[[a]]\b))",
            JsString::from("aaaa!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[[a]]a[[a]]\b))",
            JsString::from("aaaa!"),
            "dvy",
            1,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\B[[a]]a[[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[[K]]s[[a]]\b))",
            JsString::from("KſA!"),
            "divg",
            0,
            r#"{"matches":["KſA","KſA","KſA"],"index":0,"input":"KſA!","groups":{"whole":"KſA"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\b[[K]]s[[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[\P{Ll}&&A]s[[x]]\b))",
            JsString::from("Asx!"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[[\p{Any}]]\u{1f601}[[a]]\b))",
            JsString::from("😀😁a!"),
            "dvy",
            1,
            r#"{"matches":["😀😁a","😀😁a","😀😁a"],"index":0,"input":"😀😁a!","groups":{"whole":"😀😁a"},"indices":[[0,5],[0,5],[0,5]],"indexGroups":{"whole":[0,5]},"lastIndex":5,"source":"(?<whole>(\\B[[\\p{Any}]]\\u{1f601}[[a]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[\u{1f600}]]\u{1f601}[[\u{1f602}]]\B))",
            JsString::from("!😀😁😂!"),
            "dvg",
            0,
            r#"{"matches":["😀😁😂","😀😁😂","😀😁😂"],"index":1,"input":"!😀😁😂!","groups":{"whole":"😀😁😂"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\B[[\\u{1f600}]]\\u{1f601}[[\\u{1f602}]]\\B))"}"#,
        ),
        (
            r"(?<whole>(\b[[\uD800]]\uD800[[\uD800]]\B))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b[[\\uD800]]\\uD800[[\\uD800]]\\B))"}"#,
        ),
        (
            r"(?<whole>(\b[[\p{Any}]]a[[a]]\b))",
            JsString::from("Kaa!"),
            "divg",
            0,
            r#"{"matches":["Kaa","Kaa","Kaa"],"index":0,"input":"Kaa!","groups":{"whole":"Kaa"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\b[[\\p{Any}]]a[[a]]\\b))"}"#,
        ),
        (
            r"^(?<whole>(\b[[a]]b[[c]]\b))$",
            JsString::from("x\r\nabc\u{2028}de"),
            "dvmg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":3,"input":"x\r\nabc de","groups":{"whole":"abc"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"^(?<whole>(\\b[[a]]b[[c]]\\b))$"}"#,
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
fn checked_v_set_class_sequence_state_context_and_consumers() {
    check(
        r"/[a--b][[b]][[c]][[d]]/v.test('abcd')&&!/[a--b][[b]][[c]][[d]]/v.test('xbcd')&&!/[a--b][[b]][[c]][[d]]/v.test('axcd')&&!/[a--b][[b]][[c]][[d]]/v.test('abxd')&&!/[a--b][[b]][[c]][[d]]/v.test('abcx')&&!/[a--b][[b]][[c]][[d]]/v.test('abc')&&!/[a--a][[b]][[c]][[d]]/v.test('abcd')&&!/[[a]][[b]][[c]][d--d]/v.test('abcd')",
    );
    check(
        r"let ok=true;for(let n of [5,6,17]){let text='a'.repeat(n),r=new RegExp('(?<whole>('+'[[a]]'.repeat(n)+'))','dvy'),m=r.exec(text);ok=ok&&m!==null&&m.length===3&&m[0]===text&&m[1]===text&&m[2]===text&&m.groups.whole===text&&m.indices[0][0]===0&&m.indices[2][1]===n&&m.indices.groups.whole===m.indices[1]&&r.lastIndex===n;r.lastIndex=0;ok=ok&&r.exec(text.slice(1))===null&&r.lastIndex===0}ok",
    );
    check(
        r#"let marker=0;eval("marker=1; /\\b[[a]][[b]][[c]][[d]]\\b/v.test('abcd')")&&marker===1"#,
    );
    check(
        r"let a=/\B[[a]][[a]][[a]][[b]]\b/dvg,b=/\B[[a]][[a]][[a]][[ab]]\b/dvg,x=a.exec('aaaab!'),y=b.exec('aaaaa!');x.index===1&&x[0]==='aaab'&&x.indices[0][1]===5&&a.lastIndex===5&&y.index===1&&y[0]==='aaaa'&&y.indices[0][1]===5&&b.lastIndex===5",
    );
    check(
        r"let g=/\B[[a]][[a]][[a]][[b]]\b/vg,y=/\B[[a]][[a]][[a]][[b]]\b/vy,a=g.exec('aaaab!');a.index===1&&a[0]==='aaab'&&g.lastIndex===5&&g.exec('aaaab!')===null&&g.lastIndex===0&&y.exec('aaaab!')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaaab!')[0]==='aaab')&&y.lastIndex===5",
    );
    check(
        r"/\b[[a]][[b]][[c]][[d]]/v.test('abcd')&&/\B[[a]][[b]][[c]][[d]]/v.test('xabcd')&&/[[a]][[b]][[c]][[d]]\b/v.test('abcd!')&&/[[a]][[b]][[c]][[d]]\B/v.test('abcdx')&&/\b[[a]][[b]][[c]][[d]]\b/v.test('!abcd!')&&/\b[[a]][[b]][[c]][[d]]\B/v.test('!abcdx')&&/\B[[a]][[b]][[c]][[d]]\b/v.test('xabcd!')&&/\B[[a]][[b]][[c]][[d]]\B/v.test('xabcdx')",
    );
    check(
        r"!/\b[[a]][[b]][[c]][[d]]/v.test('xabcd')&&!/[[a]][[b]][[c]][[d]]\b/v.test('abcdx')&&!/\B[[a]][[b]][[c]][[d]]/v.test('!abcd')&&!/[[a]][[b]][[c]][[d]]\B/v.test('abcd!')",
    );
    check(
        r"/\b[[K]][[s]][[a]][[b]]\b/iv.test('KſAb')&&!/\b[[K]][[s]][[a]][[b]]\b/v.test('Ksab')&&!/\b[[K]][[s]][[a]][[b]]\b/v.test('Kſab')&&!/\b[[K]][[s]][[a]][[b]]\b/v.test('KsAb')&&/\b[[K]][[s]][[a]][[b]]\b/v.test('Ksab')&&/^[[^K]][[s]][[a]][[b]]$/iv.test('ſSAb')&&!/^[[^K]][[s]][[a]][[b]]$/iv.test('KSAb')",
    );
    check(
        r"/\b[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]\b/iv.test('Kabcſ!')&&!/\b[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]\b/v.test('Kabcſ!')&&/\b[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]/iv.test('Kabcſ')&&!/\b[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]/v.test('Kabcſ')&&/[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]\b/iv.test('Kabcſ!')&&!/[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]\b/v.test('Kabcſ!')",
    );
    check(
        r"/\b[\P{Ll}&&A][[s]][[x]][[a]]\b/v.test('Asxa')&&!/\b[\P{Ll}&&A][[s]][[x]][[a]]\b/iv.test('Asxa')",
    );
    check(
        r"let r=/[[\p{Any}]][[\u{1f601}]][[\u{1f602}]][[\u{1f603}]][[\u{1f604}]][[\u{1f605}]]/dvy;r.lastIndex=1;let m=r.exec('😀😁😂😃😄😅!');m.index===0&&m[0]==='😀😁😂😃😄😅'&&m.indices[0][1]===12&&r.lastIndex===12",
    );
    check(
        r"let r=/[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]][[\u{1f603}]]/dvy;r.lastIndex=1;let m=r.exec('😀😁😂😃!');m.index===0&&m[0]==='😀😁😂😃'&&m.indices[0][1]===8&&r.lastIndex===8",
    );
    check(
        r"let r=/\b[[\uD800]][[\uD800]][[\uD800]][[\uD800]][[\uD800]]\B/dv,m=r.exec('A\uD800\uD800\uD800\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800\uD800\uD800\uD800'&&m.indices[0][1]===6&&!/[[\uD83D]][[\uDE00]][[a]][[b]]/v.test('😀ab')&&!/[[a]][[\uD83D]][[\uDE00]][[b]]/v.test('a😀b')",
    );
    check(
        r"let r=/^\b[[a]][[b]][[c]][[d]]\b$/dvmg,m=r.exec('x\r\nabcd\u2028ef');m.index===3&&m[0]==='abcd'&&m.indices[0][1]===7&&r.lastIndex===7&&r.exec('x\r\nabcd\u2028ef')===null&&r.lastIndex===0&&!/^[[a]][[b]][[c]][[d]]$/v.test('abcd\n')&&/^[[a]][[b]][[c]][[d]]/v.test('abcd!')&&/[[a]][[b]][[c]][[d]]$/v.test('xabcd')",
    );
    check(
        r"let r=/(?<whole>(\B[[a]][[a]][[a]][[b]]\b))/dvg,a=[...'aaaab! aaaab!'.matchAll(r)];a.length===2&&a[0][0]==='aaab'&&a[0][1]==='aaab'&&a[0][2]==='aaab'&&a[0].index===1&&a[0].groups.whole==='aaab'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===5&&a[1].index===8&&a[1].indices[2][1]===12&&r.lastIndex===0",
    );
    check(
        r"'aaaab! aaaab!'.replace(/(?<whole>(\B[[a]][[a]][[a]][[b]]\b))/vg,'<$1:$<whole>>')==='a<aaab:aaab>! a<aaab:aaab>!'&&'aaaab!'.split(/(\B[[a]][[a]][[a]][[b]]\b)/v).join('|')==='a|aaab|!'&&'aaaab!'.match(/\B[[a]][[a]][[a]][[b]]\b/v)[0]==='aaab'",
    );
    check(
        r"let seen='',out='aaaab! aaaab!'.replace(/(?<whole>(\B[[a]][[a]][[a]][[b]]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aaab;8:aaab;'",
    );
    check(
        r"let r=/\B[[a]][[a]][[a]][[b]]\b/vg;r.lastIndex=4;'aaaab!'.search(r)===1&&r.lastIndex===4",
    );
    check(
        r"let trace='',r=/[[\p{Any}]][[\u{1f601}]][[\u{1f602}]][[\u{1f603}]][[\u{1f604}]][[\u{1f605}]]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁😂😃😄😅!'}});trace==='si'&&m.index===0&&m[0]==='😀😁😂😃😄😅'&&r.lastIndex===12",
    );
    check(
        r"let seen=0,r=/\B[[a]][[a]][[a]][[b]]\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaaab!');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\b[[K]][[s]][[a]][[b]]\b/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='\\b[[K]][[s]][[a]][[b]]\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſAb')===null&&c.lastIndex===0&&r.exec('KſAb')[0]==='KſAb'",
    );
    check(
        r"let a=/\B[[a]][[a]][[a]][[b]]\b/vy,b=/\B[[a]][[a]][[a]][[b]]\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaaab!')}catch(e){x=e instanceof TypeError}try{b.exec('aaaab!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /[a--a][[b]][[c]][d&&]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    check(r"/[a][[b]][[c]][[d]]/v.test('abcd')");
    check(
        r"let r=/[a][[b]][[c]][[d]]/v,m=r.exec('abcd');m!==null&&m.length===1&&m[0]==='abcd'&&m.index===0&&m.input==='abcd'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a][[b]][[c]][[d]]'&&r.lastIndex===0&&!r.test('xbcd')&&!r.test('axcd')&&!r.test('abxd')&&!r.test('abcx')&&!r.test('abc')&&r.exec('xabcd').index===1&&r.exec('abcdx').index===0&&r.lastIndex===0",
    );
    check(r"/[[a]][b][[c]][[d]]/v.test('abcd')");
    check(
        r"let r=/[[a]][b][[c]][[d]]/v,m=r.exec('abcd');m!==null&&m.length===1&&m[0]==='abcd'&&m.index===0&&m.input==='abcd'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][b][[c]][[d]]'&&r.lastIndex===0&&!r.test('xbcd')&&!r.test('axcd')&&!r.test('abxd')&&!r.test('abcx')&&!r.test('abc')&&r.exec('xabcd').index===1&&r.exec('abcdx').index===0&&r.lastIndex===0",
    );
    check(r"/[[a]][[b]][[c]][d]/v.test('abcd')");
    check(
        r"let r=/[[a]][[b]][[c]][d]/v,m=r.exec('abcd');m!==null&&m.length===1&&m[0]==='abcd'&&m.index===0&&m.input==='abcd'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][[b]][[c]][d]'&&r.lastIndex===0&&!r.test('xbcd')&&!r.test('axcd')&&!r.test('abxd')&&!r.test('abcx')&&!r.test('abc')&&r.exec('xabcd').index===1&&r.exec('abcdx').index===0&&r.lastIndex===0",
    );
    for source in [
        r"/a[[b]][[c]][[d]][[e]]/v.test('abcde')",
        r"/[[a]]b[[c]][[d]][[e]]/v.test('abcde')",
        r"/[[a]][[b]][[c]][[d]]e/v.test('abcde')",
        r"/[[a]][[b]][[c]]d/v.test('abcd')",
        r"/[[a]]\b[[b]][[c]][[d]]/v.test('abcd')",
        r"/[[a]][[b]][[c]]\b[[d]]/v.test('abcd')",
        r"/\b^[[a]][[b]][[c]][[d]]/v.test('abcd')",
        r"/[[a]][[b]][[c]][[d]]$\b/v.test('abcd')",
        r"/\b\b[[a]][[b]][[c]][[d]]/v.test('abcd')",
        r"/[[a]][[b]][[c]][[d]]\b\b/v.test('abcd')",
        r"/([[a]])[[b]][[c]][[d]]/v.test('abcd')",
        r"/[[a]]([[b]][[c]])[[d]]/v.test('abcd')",
        r"/[[a]][[b]][[c]]([[d]])/v.test('abcd')",
        r"/[[a]]+[[b]][[c]][[d]]/v.test('abcd')",
        r"/[[a]][[b]][[c]][[d]]?/v.test('abcd')",
        r"/(?:\b[[a]][[b]][[c]][[d]]\b)+/v.test('abcd')",
        r"/[[a]][[b]][[c]][[d]]|[[e]][[f]][[g]][[h]]/v.test('abcd')",
        r"/[a--a][[b]][[c]][[\q{d}]]/v.test('abcd')",
        r"/[[\q{a}]][[b]][[c]][d--d]/v.test('abcd')",
        r"/[a--a][[b]][[c]][[\p{RGI_Emoji}]]/v.test('abcd')",
        r"/[[\p{RGI_Emoji}]][[b]][[c]][d--d]/v.test('abcd')",
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
        .eval(r"var kept=/(?<whole>(\B[[a]][[a]][[a]][[b]]\b))/dvg;kept.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"let m=kept.exec('aaaab!');m.index===1&&m.groups.whole==='aaab'&&m.indices[2][1]===5&&kept.lastIndex===5"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/\B[[a]][[a]][[a]][[c]]\b/vg,sticky=/[[a]][[a]][[a]][[a]]/vy,marker=0")
        .unwrap();
    // The first three checked classes succeed; the fourth fails before word gates.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval(r"sticky.test(text)&&sticky.lastIndex===4"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval(r"try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r"marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn checked_v_set_class_sequence_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>(\B[[a]][[a]][[a]][[b]]\b))",
            JsString::from("aaaab!"),
            "dvg",
            0,
            r#"{"matches":["aaab","aaab","aaab"],"index":1,"input":"aaaab!","groups":{"whole":"aaab"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>(\\B[[a]][[a]][[a]][[b]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[[a]][[a]][[a]][[b]]\b))",
            JsString::from("aaaab!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[[a]][[a]][[a]][[b]]\b))",
            JsString::from("aaaab!"),
            "dvy",
            1,
            r#"{"matches":["aaab","aaab","aaab"],"index":1,"input":"aaaab!","groups":{"whole":"aaab"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>(\\B[[a]][[a]][[a]][[b]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]\b))",
            JsString::from("Kabcſ!"),
            "divg",
            0,
            r#"{"matches":["Kabcſ","Kabcſ","Kabcſ"],"index":0,"input":"Kabcſ!","groups":{"whole":"Kabcſ"},"indices":[[0,5],[0,5],[0,5]],"indexGroups":{"whole":[0,5]},"lastIndex":5,"source":"(?<whole>(\\b[[\\p{Any}]][[a]][[b]][[c]][[\\p{Any}]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[\P{Ll}&&A][[s]][[x]][[a]]\b))",
            JsString::from("Asxa!"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([[\p{Any}]][[\u{1f601}]][[\u{1f602}]][[\u{1f603}]][[\u{1f604}]][[\u{1f605}]]))",
            JsString::from("😀😁😂😃😄😅!"),
            "dvy",
            1,
            r#"{"matches":["😀😁😂😃😄😅","😀😁😂😃😄😅","😀😁😂😃😄😅"],"index":0,"input":"😀😁😂😃😄😅!","groups":{"whole":"😀😁😂😃😄😅"},"indices":[[0,12],[0,12],[0,12]],"indexGroups":{"whole":[0,12]},"lastIndex":12,"source":"(?<whole>([[\\p{Any}]][[\\u{1f601}]][[\\u{1f602}]][[\\u{1f603}]][[\\u{1f604}]][[\\u{1f605}]]))"}"#,
        ),
        (
            r"(?<whole>(\b[[\uD800]][[\uD800]][[\uD800]][[\uD800]][[\uD800]]\B))",
            JsString::from_code_units(vec![65, 0xd800, 0xd800, 0xd800, 0xd800, 0xd800, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800\ud800\ud800","\ud800\ud800\ud800\ud800\ud800","\ud800\ud800\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800\ud800\ud800"},"indices":[[1,6],[1,6],[1,6]],"indexGroups":{"whole":[1,6]},"lastIndex":6,"source":"(?<whole>(\\b[[\\uD800]][[\\uD800]][[\\uD800]][[\\uD800]][[\\uD800]]\\B))"}"#,
        ),
        (
            r"^(?<whole>(\b[[a]][[b]][[c]][[d]]\b))$",
            JsString::from("x\r\nabcd\u{2028}ef"),
            "dvmg",
            0,
            r#"{"matches":["abcd","abcd","abcd"],"index":3,"input":"x\r\nabcd ef","groups":{"whole":"abcd"},"indices":[[3,7],[3,7],[3,7]],"indexGroups":{"whole":[3,7]},"lastIndex":7,"source":"^(?<whole>(\\b[[a]][[b]][[c]][[d]]\\b))$"}"#,
        ),
        (
            r"(?<whole>(\B[[a]][[a]][[a]][[ab]]\b))",
            JsString::from("aaaaa!"),
            "dvg",
            0,
            r#"{"matches":["aaaa","aaaa","aaaa"],"index":1,"input":"aaaaa!","groups":{"whole":"aaaa"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>(\\B[[a]][[a]][[a]][[ab]]\\b))"}"#,
        ),
        (
            r"^(?<whole>([[^K]][[s]][[a]][[b]]))$",
            JsString::from("ſSAb"),
            "divy",
            0,
            r#"{"matches":["ſSAb","ſSAb","ſSAb"],"index":0,"input":"ſSAb","groups":{"whole":"ſSAb"},"indices":[[0,4],[0,4],[0,4]],"indexGroups":{"whole":[0,4]},"lastIndex":4,"source":"^(?<whole>([[^K]][[s]][[a]][[b]]))$"}"#,
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
fn flat_v_set_class_sequence_state_context_and_consumers() {
    check(
        r"/[a][b][c]/v.test('abc')&&!/[a][b][c]/v.test('xbc')&&!/[a][b][c]/v.test('axc')&&!/[a][b][c]/v.test('abx')&&!/[a][b][c]/v.test('ab')&&!/[][b][c]/v.test('abc')&&!/[a][][c]/v.test('abc')&&!/[a][b][]/v.test('abc')",
    );
    check(
        r"let ok=true;for(let n of [3,4,17]){let text='a'.repeat(n),r=new RegExp('(?<whole>('+'[a]'.repeat(n)+'))','dvy'),m=r.exec(text);ok=ok&&m!==null&&m.length===3&&m[0]===text&&m[1]===text&&m[2]===text&&m.groups.whole===text&&m.indices[0][0]===0&&m.indices[2][1]===n&&m.indices.groups.whole===m.indices[1]&&r.lastIndex===n;r.lastIndex=0;ok=ok&&r.exec(text.slice(1))===null&&r.lastIndex===0}ok",
    );
    check(r#"let marker=0;eval("marker=1; /\\b[a][b][c]\\b/v.test('abc')")&&marker===1"#);
    check(
        r"let a=/\B[a][a][b]\b/dvg,b=/\B[a][a][ab]\b/dvg,x=a.exec('aaab!'),y=b.exec('aaaa!');x.index===1&&x[0]==='aab'&&x.indices[0][1]===4&&a.lastIndex===4&&y.index===1&&y[0]==='aaa'&&y.indices[0][1]===4&&b.lastIndex===4",
    );
    check(
        r"let g=/\B[a][a][b]\b/vg,y=/\B[a][a][b]\b/vy,a=g.exec('aaab!');a.index===1&&a[0]==='aab'&&g.lastIndex===4&&g.exec('aaab!')===null&&g.lastIndex===0&&y.exec('aaab!')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaab!')[0]==='aab')&&y.lastIndex===4",
    );
    check(
        r"/\b[a][b][c]/v.test('abc')&&/\B[a][b][c]/v.test('xabc')&&/[a][b][c]\b/v.test('abc!')&&/[a][b][c]\B/v.test('abcx')&&/\b[a][b][c]\b/v.test('!abc!')&&/\b[a][b][c]\B/v.test('!abcx')&&/\B[a][b][c]\b/v.test('xabc!')&&/\B[a][b][c]\B/v.test('xabcx')",
    );
    check(
        r"!/\b[a][b][c]/v.test('xabc')&&!/[a][b][c]\b/v.test('abcx')&&!/\B[a][b][c]/v.test('!abc')&&!/[a][b][c]\B/v.test('abc!')&&/\b[a][b][c]/v.exec('abcx')[0]==='abc'",
    );
    check(
        r"/\b[K][s][a]\b/iv.test('KſA')&&!/\b[K][s][a]\b/v.test('Ksa')&&!/\b[K][s][a]\b/v.test('Kſa')&&!/\b[K][s][a]\b/v.test('KsA')&&/\b[K][s][a]\b/v.test('Ksa')&&/^[^K][s][a][b]$/iv.test('ſSAb')&&!/^[^K][s][a][b]$/iv.test('KSAb')",
    );
    check(
        r"/\b[\p{Any}][a][b][\p{Any}]\b/iv.test('Kabſ!')&&!/\b[\p{Any}][a][b][\p{Any}]\b/v.test('Kabſ!')&&/\b[\p{Any}][a][b][\p{Any}]/iv.test('Kabſ')&&!/\b[\p{Any}][a][b][\p{Any}]/v.test('Kabſ')&&/[\p{Any}][a][b][\p{Any}]\b/iv.test('Kabſ!')&&!/[\p{Any}][a][b][\p{Any}]\b/v.test('Kabſ!')",
    );
    check(r"/\b[\P{Ll}][s][a]\b/v.test('Asa')&&!/\b[\P{Ll}][s][a]\b/iv.test('Asa')");
    check(
        r"let r=/[\p{Any}][\u{1f601}][\u{1f602}]/dvy;r.lastIndex=1;let m=r.exec('😀😁😂!');m.index===0&&m[0]==='😀😁😂'&&m.indices[0][1]===6&&r.lastIndex===6",
    );
    check(
        r"let r=/[\u{1f600}][\u{1f601}][\u{1f602}][\u{1f603}]/dvy;r.lastIndex=1;let m=r.exec('😀😁😂😃!');m.index===0&&m[0]==='😀😁😂😃'&&m.indices[0][1]===8&&r.lastIndex===8",
    );
    check(
        r"let r=/\b[\uD800][\uD800][\uD800]\B/dv,m=r.exec('A\uD800\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&!/[\uD83D][\uDE00][a]/v.test('😀a')&&!/[a][\uD83D][\uDE00]/v.test('a😀')",
    );
    check(
        r"let r=/^\b[a][b][c][d]\b$/dvmg,m=r.exec('x\r\nabcd\u2028ef');m.index===3&&m[0]==='abcd'&&m.indices[0][1]===7&&r.lastIndex===7&&r.exec('x\r\nabcd\u2028ef')===null&&r.lastIndex===0&&!/^[a][b][c][d]$/v.test('abcd\n')&&/^[a][b][c][d]/v.test('abcd!')&&/[a][b][c][d]$/v.test('xabcd')",
    );
    check(
        r"let r=/(?<whole>(\B[a][a][b]\b))/dvg,a=[...'aaab! aaab!'.matchAll(r)];a.length===2&&a[0][0]==='aab'&&a[0][1]==='aab'&&a[0][2]==='aab'&&a[0].index===1&&a[0].groups.whole==='aab'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===4&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0",
    );
    check(
        r"'aaab! aaab!'.replace(/(?<whole>(\B[a][a][b]\b))/vg,'<$1:$<whole>>')==='a<aab:aab>! a<aab:aab>!'&&'aaab!'.split(/(\B[a][a][b]\b)/v).join('|')==='a|aab|!'&&'aaab!'.match(/\B[a][a][b]\b/v)[0]==='aab'",
    );
    check(
        r"let seen='',out='aaab! aaab!'.replace(/(?<whole>(\B[a][a][b]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aab;7:aab;'",
    );
    check(r"let r=/\B[a][a][b]\b/vg;r.lastIndex=4;'aaab!'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/[\p{Any}][\u{1f601}][\u{1f602}]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁😂!'}});trace==='si'&&m.index===0&&m[0]==='😀😁😂'&&r.lastIndex===6",
    );
    check(
        r"let seen=0,r=/\B[a][a][b]\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaab!');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\b[K][s][a]\b/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='\\b[K][s][a]\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſA')===null&&c.lastIndex===0&&r.exec('KſA')[0]==='KſA'",
    );
    check(
        r"let a=/\B[a][a][b]\b/vy,b=/\B[a][a][b]\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaab!')}catch(e){x=e instanceof TypeError}try{b.exec('aaab!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /[][b][c&&]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    check(r"/[[a]][b][c]/v.test('abc')");
    check(
        r"let r=/[[a]][b][c]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][b][c]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('xabc').index===1&&r.exec('abcx').index===0&&r.lastIndex===0",
    );
    check(r"/[a][[b]][c]/v.test('abc')");
    check(
        r"let r=/[a][[b]][c]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a][[b]][c]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('xabc').index===1&&r.exec('abcx').index===0&&r.lastIndex===0",
    );
    check(r"/[a][b][[c]]/v.test('abc')");
    check(
        r"let r=/[a][b][[c]]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a][b][[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('xabc').index===1&&r.exec('abcx').index===0&&r.lastIndex===0",
    );
    check(r"/[a&&a][b][c]/v.test('abc')");
    check(
        r"let r=/[a&&a][b][c]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a&&a][b][c]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('xabc').index===1&&r.exec('abcx').index===0&&r.lastIndex===0",
    );
    check(r"/[a][b][c--d]/v.test('abc')");
    check(
        r"let r=/[a][b][c--d]/v,m=r.exec('abc');m!==null&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a][b][c--d]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('xabc').index===1&&r.exec('abcx').index===0&&r.lastIndex===0",
    );
    for source in [
        r"/a[a][b][c]/v.test('aabc')",
        r"/[a]b[c][d]/v.test('abcd')",
        r"/[a][b][c]d/v.test('abcd')",
        r"/[a]\b[b][c]/v.test('abc')",
        r"/\b^[a][b][c]/v.test('abc')",
        r"/[a][b][c]$\b/v.test('abc')",
        r"/\b\b[a][b][c]/v.test('abc')",
        r"/[a][b][c]\b\b/v.test('abc')",
        r"/([a])[b][c]/v.test('abc')",
        r"/[a]([b])[c]/v.test('abc')",
        r"/[a][b]([c])/v.test('abc')",
        r"/[a]+[b][c]/v.test('abc')",
        r"/[a][b][c]?/v.test('abc')",
        r"/(?:\b[a][b][c]\b)+/v.test('abc')",
        r"/[a][b][c]|[d][e][f]/v.test('abc')",
        r"/[][b][\q{c}]/v.test('abc')",
        r"/[\q{a}][b][]/v.test('abc')",
        r"/[][b][\p{RGI_Emoji}]/v.test('abc')",
        r"/[\p{RGI_Emoji}][b][]/v.test('abc')",
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
        .eval(r"var kept=/(?<whole>(\B[a][a][b]\b))/dvg;kept.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=kept.exec('aaab!');m.index===1&&m.groups.whole==='aab'&&m.indices[2][1]===4&&kept.lastIndex===4"), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/\B[a][a][c]\b/vg,sticky=/[a][a][a]/vy,marker=0")
        .unwrap();
    // The first two flat classes succeed; the third fails before word gates.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval(r"sticky.test(text)&&sticky.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval(r"try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r"marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn flat_v_set_class_sequence_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>(\B[a][a][b]\b))",
            JsString::from("aaab!"),
            "dvg",
            0,
            r#"{"matches":["aab","aab","aab"],"index":1,"input":"aaab!","groups":{"whole":"aab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\B[a][a][b]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[a][a][b]\b))",
            JsString::from("aaab!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[a][a][b]\b))",
            JsString::from("aaab!"),
            "dvy",
            1,
            r#"{"matches":["aab","aab","aab"],"index":1,"input":"aaab!","groups":{"whole":"aab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\B[a][a][b]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[\p{Any}][a][b][\p{Any}]\b))",
            JsString::from("Kabſ!"),
            "divg",
            0,
            r#"{"matches":["Kabſ","Kabſ","Kabſ"],"index":0,"input":"Kabſ!","groups":{"whole":"Kabſ"},"indices":[[0,4],[0,4],[0,4]],"indexGroups":{"whole":[0,4]},"lastIndex":4,"source":"(?<whole>(\\b[\\p{Any}][a][b][\\p{Any}]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[\P{Ll}][s][a]\b))",
            JsString::from("Asa!"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}]))",
            JsString::from("😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀!"),
            "dvy",
            1,
            r#"{"matches":["😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀","😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀","😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀"],"index":0,"input":"😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀!","groups":{"whole":"😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀"},"indices":[[0,34],[0,34],[0,34]],"indexGroups":{"whole":[0,34]},"lastIndex":34,"source":"(?<whole>([\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}][\\p{Any}]))"}"#,
        ),
        (
            r"(?<whole>(\b[\uD800][\uD800][\uD800]\B))",
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b[\\uD800][\\uD800][\\uD800]\\B))"}"#,
        ),
        (
            r"^(?<whole>(\b[a][b][c][d]\b))$",
            JsString::from("x\r\nabcd ef"),
            "dvmg",
            0,
            r#"{"matches":["abcd","abcd","abcd"],"index":3,"input":"x\r\nabcd ef","groups":{"whole":"abcd"},"indices":[[3,7],[3,7],[3,7]],"indexGroups":{"whole":[3,7]},"lastIndex":7,"source":"^(?<whole>(\\b[a][b][c][d]\\b))$"}"#,
        ),
        (
            r"(?<whole>(\B[a][a][ab]\b))",
            JsString::from("aaaa!"),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\B[a][a][ab]\\b))"}"#,
        ),
        (
            r"^(?<whole>([^K][s][a][b]))$",
            JsString::from("ſSAb"),
            "divy",
            0,
            r#"{"matches":["ſSAb","ſSAb","ſSAb"],"index":0,"input":"ſSAb","groups":{"whole":"ſSAb"},"indices":[[0,4],[0,4],[0,4]],"indexGroups":{"whole":[0,4]},"lastIndex":4,"source":"^(?<whole>([^K][s][a][b]))$"}"#,
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
fn mixed_v_set_class_sequence_state_context_and_consumers() {
    check(
        r"let ok=true;for(let source of ['[a][[b]][c]','[[a]][b][c]','[a][b][[c]]','[[a]][[b]][c]','[[a]][b][[c]]','[a][[b]][[c]]']){let r=new RegExp(source,'dv'),m=r.exec('abc');ok=ok&&m!==null&&m[0]==='abc'&&m.index===0&&m.indices[0][1]===3&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')}ok",
    );
    check(
        r"let ok=true;for(let n of [3,4,17]){let text='a'.repeat(n),r=new RegExp('(?<whole>([a]'+'[[a]]'.repeat(n-1)+'))','dvy'),m=r.exec(text);ok=ok&&m!==null&&m.length===3&&m[0]===text&&m[1]===text&&m[2]===text&&m.groups.whole===text&&m.indices[0][0]===0&&m.indices[2][1]===n&&m.indices.groups.whole===m.indices[1]&&r.lastIndex===n;r.lastIndex=0;ok=ok&&r.exec(text.slice(1))===null&&r.lastIndex===0}ok",
    );
    check(r#"let marker=0;eval("marker=1; /\\b[a][[b]][c]\\b/v.test('abc')")&&marker===1"#);
    check(
        r"let a=/\B[a][[a]][b]\b/dvg,b=/\B[[a]][a][[ab]]\b/dvg,x=a.exec('aaab!'),y=b.exec('aaaa!');x.index===1&&x[0]==='aab'&&x.indices[0][1]===4&&a.lastIndex===4&&y.index===1&&y[0]==='aaa'&&y.indices[0][1]===4&&b.lastIndex===4",
    );
    check(
        r"let g=/\B[a][[a]][b]\b/vg,y=/\B[a][[a]][b]\b/vy,a=g.exec('aaab!');a.index===1&&a[0]==='aab'&&g.lastIndex===4&&g.exec('aaab!')===null&&g.lastIndex===0&&y.exec('aaab!')===null&&y.lastIndex===0&&(y.lastIndex=1,y.exec('aaab!')[0]==='aab')&&y.lastIndex===4",
    );
    check(
        r"/\b[a][[b]][c]/v.test('abc')&&/\B[a][[b]][c]/v.test('xabc')&&/[a][[b]][c]\b/v.test('abc!')&&/[a][[b]][c]\B/v.test('abcx')&&/\b[a][[b]][c]\b/v.test('!abc!')&&/\b[a][[b]][c]\B/v.test('!abcx')&&/\B[a][[b]][c]\b/v.test('xabc!')&&/\B[a][[b]][c]\B/v.test('xabcx')",
    );
    check(
        r"!/\b[a][[b]][c]/v.test('xabc')&&!/[a][[b]][c]\b/v.test('abcx')&&!/\B[a][[b]][c]/v.test('!abc')&&!/[a][[b]][c]\B/v.test('abc!')&&/\b[a][[b]][c]/v.exec('abcx')[0]==='abc'",
    );
    check(
        r"/\b[K][[s]][a]\b/iv.test('KſA')&&!/\b[K][[s]][a]\b/v.test('Ksa')&&!/\b[K][[s]][a]\b/v.test('Kſa')&&!/\b[K][[s]][a]\b/v.test('KsA')&&/\b[K][[s]][a]\b/v.test('Ksa')&&/^[^K][[s]][a][[b]]$/iv.test('ſSAb')&&!/^[^K][[s]][a][[b]]$/iv.test('KSAb')",
    );
    check(
        r"/\b[\p{Any}][[a]][b][[\p{Any}]]\b/iv.test('Kabſ!')&&!/\b[\p{Any}][[a]][b][[\p{Any}]]\b/v.test('Kabſ!')&&/\b[\p{Any}][[a]][b][[\p{Any}]]/iv.test('Kabſ')&&!/\b[\p{Any}][[a]][b][[\p{Any}]]/v.test('Kabſ')&&/[\p{Any}][[a]][b][[\p{Any}]]\b/iv.test('Kabſ!')&&!/[\p{Any}][[a]][b][[\p{Any}]]\b/v.test('Kabſ!')",
    );
    check(
        r"/\b[\P{Ll}&&A][s][[a]]\b/v.test('Asa')&&!/\b[\P{Ll}&&A][s][[a]]\b/iv.test('Asa')&&!/[][[b]][[c]]/v.test('abc')&&!/[a--a][b][c]/v.test('abc')&&!/[[a]][][c]/v.test('abc')&&!/[a][b--b][c]/v.test('abc')&&!/[a][[b]][]/v.test('abc')",
    );
    check(
        r"let r=/[\p{Any}][[\u{1f601}]][\u{1f602}]/dvy;r.lastIndex=1;let m=r.exec('😀😁😂!');m.index===0&&m[0]==='😀😁😂'&&m.indices[0][1]===6&&r.lastIndex===6",
    );
    check(
        r"let r=/[[\u{1f600}]][\u{1f601}][[\u{1f602}]][\u{1f603}]/dvy;r.lastIndex=1;let m=r.exec('😀😁😂😃!');m.index===0&&m[0]==='😀😁😂😃'&&m.indices[0][1]===8&&r.lastIndex===8",
    );
    check(
        r"let r=/\b[\uD800][[\uD800]][\uD800]\B/dv,m=r.exec('A\uD800\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&!/[\uD83D][[\uDE00]][a]/v.test('😀a')&&!/[[a]][\uD83D][\uDE00]/v.test('a😀')",
    );
    check(
        r"let r=/^\b[[a]][b][[c]][d]\b$/dvmg,m=r.exec('x\r\nabcd\u2028ef');m.index===3&&m[0]==='abcd'&&m.indices[0][1]===7&&r.lastIndex===7&&r.exec('x\r\nabcd\u2028ef')===null&&r.lastIndex===0&&!/^[[a]][b][[c]][d]$/v.test('abcd\n')&&/^[[a]][b][[c]][d]/v.test('abcd!')&&/[[a]][b][[c]][d]$/v.test('xabcd')",
    );
    check(
        r"let r=/(?<whole>(\B[a][[a]][b]\b))/dvg,a=[...'aaab! aaab!'.matchAll(r)];a.length===2&&a[0][0]==='aab'&&a[0][1]==='aab'&&a[0][2]==='aab'&&a[0].index===1&&a[0].groups.whole==='aab'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===4&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0",
    );
    check(
        r"'aaab! aaab!'.replace(/(?<whole>(\B[a][[a]][b]\b))/vg,'<$1:$<whole>>')==='a<aab:aab>! a<aab:aab>!'&&'aaab!'.split(/(\B[a][[a]][b]\b)/v).join('|')==='a|aab|!'&&'aaab!'.match(/\B[a][[a]][b]\b/v)[0]==='aab'",
    );
    check(
        r"let seen='',out='aaab! aaab!'.replace(/(?<whole>(\B[a][[a]][b]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aab;7:aab;'",
    );
    check(r"let r=/\B[a][[a]][b]\b/vg;r.lastIndex=4;'aaab!'.search(r)===1&&r.lastIndex===4");
    check(
        r"let trace='',r=/[\p{Any}][[\u{1f601}]][\u{1f602}]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁😂!'}});trace==='si'&&m.index===0&&m[0]==='😀😁😂'&&r.lastIndex===6",
    );
    check(
        r"let seen=0,r=/\B[a][[a]][b]\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaab!');m.index===1&&seen===1&&typeof r.lastIndex==='object'",
    );
    check(
        r"let r=/\b[K][[s]][a]\b/iv,c=new RegExp(r,'dvy');c.source===r.source&&r.source==='\\b[K][[s]][a]\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſA')===null&&c.lastIndex===0&&r.exec('KſA')[0]==='KſA'",
    );
    check(
        r"let a=/\B[a][[a]][b]\b/vy,b=/\B[a][[a]][b]\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaab!')}catch(e){x=e instanceof TypeError}try{b.exec('aaab!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0",
    );
    check(
        r"let marker=0,caught=false;try{eval('marker=1; /[][[b]][c&&]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0",
    );
    for source in [
        r"/a[a][[b]][c]/v.test('aabc')",
        r"/[a]b[[c]][d]/v.test('abcd')",
        r"/[a][[b]][c]d/v.test('abcd')",
        r"/[a]\b[[b]][c]/v.test('abc')",
        r"/[a]^[[b]][c]/v.test('abc')",
        r"/[a][[b]]$[c]/v.test('abc')",
        r"/\b^[a][[b]][c]/v.test('abc')",
        r"/[a][[b]][c]$\b/v.test('abc')",
        r"/\b\b[a][[b]][c]/v.test('abc')",
        r"/[a][[b]][c]\b\b/v.test('abc')",
        r"/([a])[[b]][c]/v.test('abc')",
        r"/[a]([[b]])[c]/v.test('abc')",
        r"/[a][[b]]([c])/v.test('abc')",
        r"/[a]+[[b]][c]/v.test('abc')",
        r"/[a][[b]][c]?/v.test('abc')",
        r"/(?:\b[a][[b]][c]\b)+/v.test('abc')",
        r"/[a][[b]][c]|[[d]][e][f]/v.test('abc')",
        r"/[][[b]][\q{c}]/v.test('abc')",
        r"/[a--a][b][\q{c}]/v.test('abc')",
        r"/[\q{a}][b][[c]]/v.test('abc')",
        r"/[][[b]][\p{RGI_Emoji}]/v.test('abc')",
        r"/[a--a][b][\p{RGI_Emoji}]/v.test('abc')",
        r"/[[\p{RGI_Emoji}]][b][]/v.test('abc')",
        r"/[a&&\q{a}][b][[c]]/v.test('abc')",
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
        .eval(r"var kept=/(?<whole>(\B[a][[a]][b]\b))/dvg;kept.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=kept.exec('aaab!');m.index===1&&m.groups.whole==='aab'&&m.indices[2][1]===4&&kept.lastIndex===4"), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/\B[a][[a]][c]\b/vg,sticky=/[a][[a]][a]/vy,marker=0")
        .unwrap();
    // The first two mixed classes succeed; the third fails before word gates.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval(r"sticky.test(text)&&sticky.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval(r"try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r"marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn mixed_v_set_class_sequence_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r"(?<whole>(\B[a][[a]][b]\b))",
            JsString::from("aaab!"),
            "dvg",
            0,
            r#"{"matches":["aab","aab","aab"],"index":1,"input":"aaab!","groups":{"whole":"aab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\B[a][[a]][b]\\b))"}"#,
        ),
        (
            r"(?<whole>(\B[a][[a]][b]\b))",
            JsString::from("aaab!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>(\B[a][[a]][b]\b))",
            JsString::from("aaab!"),
            "dvy",
            1,
            r#"{"matches":["aab","aab","aab"],"index":1,"input":"aaab!","groups":{"whole":"aab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\B[a][[a]][b]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[\p{Any}][[a]][b][[\p{Any}]]\b))",
            JsString::from("Kabſ!"),
            "divg",
            0,
            r#"{"matches":["Kabſ","Kabſ","Kabſ"],"index":0,"input":"Kabſ!","groups":{"whole":"Kabſ"},"indices":[[0,4],[0,4],[0,4]],"indexGroups":{"whole":[0,4]},"lastIndex":4,"source":"(?<whole>(\\b[\\p{Any}][[a]][b][[\\p{Any}]]\\b))"}"#,
        ),
        (
            r"(?<whole>(\b[\P{Ll}&&A][s][[a]]\b))",
            JsString::from("Asa!"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r"(?<whole>([\p{Any}][[\p{Any}]][\p{Any}][[\p{Any}]][\p{Any}][[\p{Any}]][\p{Any}][[\p{Any}]][\p{Any}][[\p{Any}]][\p{Any}][[\p{Any}]][\p{Any}][[\p{Any}]][\p{Any}][[\p{Any}]][\p{Any}]))",
            JsString::from("😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀!"),
            "dvy",
            1,
            r#"{"matches":["😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀","😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀","😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀"],"index":0,"input":"😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀!","groups":{"whole":"😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀"},"indices":[[0,34],[0,34],[0,34]],"indexGroups":{"whole":[0,34]},"lastIndex":34,"source":"(?<whole>([\\p{Any}][[\\p{Any}]][\\p{Any}][[\\p{Any}]][\\p{Any}][[\\p{Any}]][\\p{Any}][[\\p{Any}]][\\p{Any}][[\\p{Any}]][\\p{Any}][[\\p{Any}]][\\p{Any}][[\\p{Any}]][\\p{Any}][[\\p{Any}]][\\p{Any}]))"}"#,
        ),
        (
            r"(?<whole>(\b[\uD800][[\uD800]][\uD800]\B))",
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b[\\uD800][[\\uD800]][\\uD800]\\B))"}"#,
        ),
        (
            r"^(?<whole>(\b[[a]][b][[c]][d]\b))$",
            JsString::from("x\r\nabcd ef"),
            "dvmg",
            0,
            r#"{"matches":["abcd","abcd","abcd"],"index":3,"input":"x\r\nabcd ef","groups":{"whole":"abcd"},"indices":[[3,7],[3,7],[3,7]],"indexGroups":{"whole":[3,7]},"lastIndex":7,"source":"^(?<whole>(\\b[[a]][b][[c]][d]\\b))$"}"#,
        ),
        (
            r"(?<whole>(\B[[a]][a][[ab]]\b))",
            JsString::from("aaaa!"),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\B[[a]][a][[ab]]\\b))"}"#,
        ),
        (
            r"^(?<whole>([^K][[s]][a][[b]]))$",
            JsString::from("ſSAb"),
            "divy",
            0,
            r#"{"matches":["ſSAb","ſSAb","ſSAb"],"index":0,"input":"ſSAb","groups":{"whole":"ſSAb"},"indices":[[0,4],[0,4],[0,4]],"indexGroups":{"whole":[0,4]},"lastIndex":4,"source":"^(?<whole>([^K][[s]][a][[b]]))$"}"#,
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
fn flat_v_set_class_literal_state_context_and_consumers() {
    check(r#"/\b[a]b\b/v.test('ab')"#);
    check(
        r#"let r=/\b[a]b\b/v,m=r.exec('ab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='ab'&&m.index===0&&m.input==='ab'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[a]b\\b'&&r.lastIndex===0&&r.exec('xb')===null&&r.exec('ax')===null&&r.exec('a')===null&&r.exec('xab')===null&&r.exec('!ab').index===1&&r.exec('abx')===null"#,
    );
    check(r#"/\b[a]b/v.test('ab')"#);
    check(
        r#"let r=/\b[a]b/v,m=r.exec('ab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='ab'&&m.index===0&&m.input==='ab'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[a]b'&&r.lastIndex===0&&r.exec('xb')===null&&r.exec('ax')===null&&r.exec('a')===null&&r.exec('xab')===null&&r.exec('!ab').index===1&&r.exec('abcx')[0]==='ab'"#,
    );
    check(r#"/[a]b\b/v.test('ab')"#);
    check(
        r#"let r=/[a]b\b/v,m=r.exec('ab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='ab'&&m.index===0&&m.input==='ab'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a]b\\b'&&r.lastIndex===0&&r.exec('xb')===null&&r.exec('ax')===null&&r.exec('a')===null&&r.exec('xab').index===1&&r.exec('abx')===null"#,
    );
    check(
        r#"let r=/(?<whole>([a]b))/dvg,m=r.exec('aaab!');m!==null&&m.length===3&&m[0]==='ab'&&m[1]==='ab'&&m[2]==='ab'&&m.index===2&&m.input==='aaab!'&&m.groups.whole==='ab'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][0]===2&&m.indices[2][1]===4&&r.lastIndex===4&&r.source==='(?<whole>([a]b))'&&r.exec('aaab!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[a]b/dvy;r.exec('aaab!')===null&&r.lastIndex===0&&(r.lastIndex=2,r.exec('aaab!')[0]==='ab')&&r.lastIndex===4&&r.exec('aaab!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[K]s/iv,m=r.exec('Kſ!');m!==null&&m[0]==='Kſ'&&m.index===0&&/[K]s/iv.test('ks')&&!/[K]s/v.test('Ks')&&!/[K]s/v.test('Kſ')&&!/[K]s/iv.test('Ka')&&!/[K]s/iv.test('as')"#,
    );
    check(
        r#"/[\P{Ll}]a/v.test('Aa')&&!/[\P{Ll}]a/iv.test('Aa')&&!/[]b/v.test('ab')&&!/[]b/v.test('')&&/[^a]b/v.test('xb')&&!/[^a]b/v.test('ab')"#,
    );
    check(
        r#"let r=/[\p{Any}]\u{1f601}/dvy;r.lastIndex=1;let a=r.exec('😀😁😀😁!'),b=r.exec('😀😁😀😁!');a.index===0&&a[0]==='😀😁'&&a.indices[0][1]===4&&b.index===4&&r.lastIndex===8&&r.exec('😀😁😀😁!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[\uD800]\uD800/dv,m=r.exec('A\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800'&&m.indices[0][1]===3&&/[\uD83D]\uDE00/v.exec('😀')===null"#,
    );
    check(
        r#"/[\]]\$/v.exec('x]$!')[0]===']$'&&/^[\u{61}]\u{62}$/v.test('ab')&&/^[\u0061]\u0062$/v.test('ab')&&!/^[\]]\$$/v.test(']$\n')"#,
    );
    check(
        r#"let r=/^(?<whole>([a]b))$/dvmg,m=r.exec('x\r\nab\u2028ac');m.index===3&&m[0]==='ab'&&m.indices[2][0]===3&&m.indices[2][1]===5&&r.lastIndex===5&&r.exec('x\r\nab\u2028ac')===null&&r.lastIndex===0&&!/^[a]b$/v.test('ab\n')&&/^[a]b/v.test('ab!')&&/[a]b$/v.test('xab')"#,
    );
    check(
        r#"let r=/(?<whole>([a]b))/dvg,a=[...'aaab! aaab!'.matchAll(r)];a.length===2&&a[0][0]==='ab'&&a[0][1]==='ab'&&a[0][2]==='ab'&&a[0].index===2&&a[0].groups.whole==='ab'&&a[0].indices.groups.whole===a[0].indices[1]&&a[1].index===8&&a[1].indices[2][1]===10&&r.lastIndex===0"#,
    );
    check(
        r#"'aaab! aaab!'.replace(/(?<whole>([a]b))/vg,'<$1:$<whole>>')==='aa<ab:ab>! aa<ab:ab>!'&&'aaab!'.split(/([a]b)/v).join('|')==='aa|ab|!'&&'aaab!'.match(/[a]b/v)[0]==='ab'"#,
    );
    check(
        r#"let seen='',out='aaab! aaab!'.replace(/(?<whole>([a]b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aaX! aaX!'&&seen==='2:ab;8:ab;'"#,
    );
    check(r#"let r=/[a]b/vg;r.lastIndex=4;'aaab!'.search(r)===2&&r.lastIndex===4"#);
    check(
        r#"let trace='',r=/[\p{Any}]\u{1f601}/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁!'}});trace==='si'&&m.index===0&&m[0]==='😀😁'&&r.lastIndex===4"#,
    );
    check(
        r#"let seen=0,r=/[a]b/v;r.lastIndex={valueOf(){seen++;return 2}};let m=r.exec('aaab!');m.index===2&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[K]s/iv,c=new RegExp(r,'dvy');c.source==='[K]s'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſ')===null&&c.lastIndex===0&&r.exec('Kſ')[0]==='Kſ'"#,
    );
    check(
        r#"let a=/[a]b/vy,b=/[a]b/vy;a.lastIndex=2;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaab!')}catch(e){x=e instanceof TypeError}try{b.exec('aaab!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===2&&b.lastIndex===0"#,
    );
    check(
        r#"let marker=0,caught=false;try{eval('marker=1; /[]\\u{110000}/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0"#,
    );
    check(
        r#"let marker=0,result,caught=false;try{result=eval("marker=1; /[a]b/v.test('ab')")}catch{caught=true}finally{marker=3}!caught&&marker===3&&result&&!/[a]b/v.test('xb')&&!/[a]b/v.test('ax')&&!/[a]b/v.test('a')"#,
    );
    check(r#"/[a]b[c]/v.test('abc')"#);
    check(
        r#"let r=/[a]b[c]/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a]b[c]'&&r.lastIndex===0&&r.exec('xbc')===null&&r.exec('axc')===null&&r.exec('abd')===null&&r.exec('ab')===null&&r.exec('xabc').index===1&&r.exec('abcx')[0]==='abc'"#,
    );
    assert_eq!(
        Realm::default().eval(r#"/[\q{a}]b/v.test('ab')"#),
        Ok(Value::Boolean(true))
    );
    for source in [
        r#"/[a]\bb/v.test('ab')"#,
        r#"/[a]^b/v.test('ab')"#,
        r#"/[a]$b/v.test('ab')"#,
        r#"/[a]bc/v.test('abc')"#,
        r#"/x[a]b/v.test('xab')"#,
        r#"/[a]+b/v.test('ab')"#,
        r#"/[a]b+/v.test('ab')"#,
        r#"/[a]b?/v.test('ab')"#,
        r#"/([a])b/v.test('ab')"#,
        r#"/[a](b)/v.test('ab')"#,
        r#"/([a]b)+/v.test('ab')"#,
        r#"/[a]b|c/v.test('ab')"#,
        r#"/[\p{RGI_Emoji}]b/v.test('ab')"#,
        r#"/[]\p{RGI_Emoji}/v.test('')"#,
        r#"/[a]b/u.test('ab')"#,
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
        .eval(r#"var kept=/(?<whole>([a]b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"let m=kept.exec('aaab!');m.index===2&&m.groups.whole==='ab'&&m.indices[2][1]===4&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r#"var r=/[a]b/vg,sticky=/[a]a/vy,marker=0"#)
        .unwrap();
    // Each flat class succeeds while the literal suffix fails. The quota is explicitly opted in.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval(r#"sticky.test(text)&&sticky.lastIndex===2"#),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn flat_v_set_class_literal_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full JSON and original spans are independently hand-derived finite expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([a]b))"#,
            JsString::from("aaab!"),
            "dvg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":2,"input":"aaab!","groups":{"whole":"ab"},"indices":[[2,4],[2,4],[2,4]],"indexGroups":{"whole":[2,4]},"lastIndex":4,"source":"(?<whole>([a]b))"}"#,
        ),
        (
            r#"(?<whole>([a]b))"#,
            JsString::from("aaab!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([a]b))"#,
            JsString::from("aaab!"),
            "dvy",
            2,
            r#"{"matches":["ab","ab","ab"],"index":2,"input":"aaab!","groups":{"whole":"ab"},"indices":[[2,4],[2,4],[2,4]],"indexGroups":{"whole":[2,4]},"lastIndex":4,"source":"(?<whole>([a]b))"}"#,
        ),
        (
            r#"(?<whole>([K]s))"#,
            JsString::from("Kſ!"),
            "divg",
            0,
            r#"{"matches":["Kſ","Kſ","Kſ"],"index":0,"input":"Kſ!","groups":{"whole":"Kſ"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>([K]s))"}"#,
        ),
        (
            r#"(?<whole>([\P{Ll}]a))"#,
            JsString::from("Aa!"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([\p{Any}]\u{1f601}))"#,
            JsString::from("😀😁!"),
            "dvy",
            1,
            r#"{"matches":["😀😁","😀😁","😀😁"],"index":0,"input":"😀😁!","groups":{"whole":"😀😁"},"indices":[[0,4],[0,4],[0,4]],"indexGroups":{"whole":[0,4]},"lastIndex":4,"source":"(?<whole>([\\p{Any}]\\u{1f601}))"}"#,
        ),
        (
            r#"(?<whole>([\uD800]\uD800))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800!","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([\\uD800]\\uD800))"}"#,
        ),
        (
            r#"^(?<whole>([a]b))$"#,
            JsString::from("x\r\nab ac"),
            "dvmg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":3,"input":"x\r\nab ac","groups":{"whole":"ab"},"indices":[[3,5],[3,5],[3,5]],"indexGroups":{"whole":[3,5]},"lastIndex":5,"source":"^(?<whole>([a]b))$"}"#,
        ),
        (
            r#"(?<whole>([\]]\$))"#,
            JsString::from("x]$!"),
            "dvg",
            0,
            r#"{"matches":["]$","]$","]$"],"index":1,"input":"x]$!","groups":{"whole":"]$"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([\\]]\\$))"}"#,
        ),
        (
            r#"^(?<whole>([\u{61}]\u{62}))$"#,
            JsString::from("ab"),
            "dvg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":0,"input":"ab","groups":{"whole":"ab"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"^(?<whole>([\\u{61}]\\u{62}))$"}"#,
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
fn flat_v_set_class_literal_word_state_context_and_consumers() {
    check(r#"/\ba[b]\b/v.test('ab')"#);
    check(
        r#"let r=/\ba[b]\b/v,m=r.exec('ab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='ab'&&m.index===0&&m.input==='ab'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\ba[b]\\b'&&r.lastIndex===0&&r.exec('xb')===null&&r.exec('ac')===null&&r.exec('a')===null&&r.exec('xab')===null&&r.exec('abx')===null&&r.exec('!ab!').index===1"#,
    );
    check(
        r#"let r=/(?<whole>(\B[a]a\b))/dvg,m=r.exec('aaa!');m!==null&&m.length===3&&m[0]==='aa'&&m[1]==='aa'&&m[2]==='aa'&&m.index===1&&m.input==='aaa!'&&m.groups.whole==='aa'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][0]===1&&m.indices[2][1]===3&&r.lastIndex===3&&r.exec('aaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\B[a]a\b/dvy;r.exec('aaa!')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aaa!')[0]==='aa')&&r.lastIndex===3&&r.exec('aaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/\b[\p{Any}]s\b/iv.test('Kſ!')&&!/\b[\p{Any}]s\b/v.test('Kſ!')&&/\b[K]s\b/iv.exec('Kſ!')[0]==='Kſ'&&!/\b[K]s\b/v.test('Ks!')&&!/\b[K]s\b/v.test('Kſ!')&&!/\b[K]s\b/iv.test('Kas!')"#,
    );
    check(
        r#"/\b[\P{Ll}]a\b/v.test('Aa!')&&!/\b[\P{Ll}]a\b/iv.test('Aa!')&&!/\B[]b\B/v.test('ab')&&!/\B[]b\B/v.test('')&&/\b[^a]b\b/v.test('xb!')&&!/\b[^a]b\b/v.test('ab!')"#,
    );
    check(
        r#"let r=/\b[\p{Any}]\u{1f601}\B/dvy;r.lastIndex=2;let m=r.exec('A😀😁!');m!==null&&m.index===1&&m[0]==='😀😁'&&m.indices[0][0]===1&&m.indices[0][1]===5&&r.lastIndex===5&&r.exec('A😀😁!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\b[\uD800]\uD800\B/dv,m=r.exec('A\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800'&&m.indices[0][1]===3&&/\b[\uD83D]\uDE00\B/v.exec('A😀!')===null"#,
    );
    check(
        r#"/\b[\]]\$\B/v.exec('x]$!')[0]===']$'&&/^\b[\u{61}]\u{62}\b$/v.test('ab')&&/^\b[\u0061]\u0062\b$/v.test('ab')&&!/^\b[a]b\b$/v.test('ab\n')"#,
    );
    check(
        r#"let r=/^(?<whole>(\b[a]b\b))$/dvmg,m=r.exec('x\r\nab\u2028ac');m.index===3&&m[0]==='ab'&&m.indices[2][0]===3&&m.indices[2][1]===5&&r.lastIndex===5&&r.exec('x\r\nab\u2028ac')===null&&r.lastIndex===0&&/^\b[a]b\b/v.test('ab!')&&/\b[a]b\b$/v.test('!ab')"#,
    );
    check(
        r#"let r=/(?<whole>(\B[a]a\b))/dvg,a=[...'aaa! aaa!'.matchAll(r)];a.length===2&&a[0][0]==='aa'&&a[0][1]==='aa'&&a[0][2]==='aa'&&a[0].index===1&&a[0].groups.whole==='aa'&&a[0].indices.groups.whole===a[0].indices[1]&&a[1].index===6&&a[1].indices[2][1]===8&&r.lastIndex===0"#,
    );
    check(
        r#"'aaa! aaa!'.replace(/(?<whole>(\B[a]a\b))/vg,'<$1:$<whole>>')==='a<aa:aa>! a<aa:aa>!'&&'aaa!'.split(/(\B[a]a\b)/v).join('|')==='a|aa|!'&&'aaa!'.match(/\B[a]a\b/v)[0]==='aa'"#,
    );
    check(
        r#"let seen='',out='aaa! aaa!'.replace(/(?<whole>(\B[a]a\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aa;6:aa;'"#,
    );
    check(r#"let r=/\B[a]a\b/vg;r.lastIndex=4;'aaa!'.search(r)===1&&r.lastIndex===4"#);
    check(
        r#"let trace='',r=/\b[\p{Any}]\u{1f601}\B/vy;r.lastIndex={valueOf(){trace+='i';return 2}};let m=r.exec({toString(){trace+='s';return 'A😀😁'}});trace==='si'&&m.index===1&&m[0]==='😀😁'&&r.lastIndex===5"#,
    );
    check(
        r#"let seen=0,r=/\B[a]a\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaa!');m.index===1&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/\b[K]s\b/iv,c=new RegExp(r,'dvy');c.source==='\\b[K]s\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſ')===null&&c.lastIndex===0&&r.exec('Kſ')[0]==='Kſ'"#,
    );
    check(
        r#"let a=/\B[a]a\b/vy,b=/\B[a]a\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaa!')}catch(e){x=e instanceof TypeError}try{b.exec('aaa!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let marker=0,caught=false;try{eval('marker=1; /\\B[]\\u{110000}\\B/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0"#,
    );
    check(
        r#"let marker=0,result,caught=false;try{result=eval("marker=1; /\\b[a]b\\b/v.test('ab')")}catch{caught=true}finally{marker=3}!caught&&marker===3&&result&&!/\b[a]b\b/v.test('xb')&&!/\b[a]b\b/v.test('ax')&&!/\b[a]b\b/v.test('a')&&!/\b[a]b\b/v.test('abx')"#,
    );
    check(
        r#"/\b[a]b/v.exec('abcx')[0]==='ab'&&/\b[a]b/v.exec('!ab').index===1&&!/\b[a]b/v.test('xab')&&/[a]b\b/v.exec('xab').index===1&&!/[a]b\b/v.test('abx')&&/\b[[a]]b\b/v.test('ab')&&/\ba[[b]]\b/v.test('ab')"#,
    );
    check(r#"/\b[a]b[c]\b/v.test('abc')"#);
    check(
        r#"let r=/\b[a]b[c]\b/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[a]b[c]\\b'&&r.lastIndex===0&&r.exec('xbc')===null&&r.exec('axc')===null&&r.exec('abd')===null&&r.exec('ab')===null&&r.exec('xabc')===null&&r.exec('abcx')===null&&r.exec('!abc!').index===1"#,
    );
    assert_eq!(
        Realm::default().eval(r#"/\b[\q{a}]b\b/v.test('ab')"#),
        Ok(Value::Boolean(true))
    );
    for source in [
        r#"/\b[a]bc\b/v.test('abc')"#,
        r#"/\bx[a]b\b/v.test('xab')"#,
        r#"/\b([a])b\b/v.test('ab')"#,
        r#"/\b[a](b)\b/v.test('ab')"#,
        r#"/(\b[a]b\b)+/v.test('ab')"#,
        r#"/\b[a]+b\b/v.test('ab')"#,
        r#"/\b[a]b+\b/v.test('ab')"#,
        r#"/\b[a]b?\b/v.test('ab')"#,
        r#"/\b[a]b\b|c/v.test('ab')"#,
        r#"/\b[a]\bb\b/v.test('ab')"#,
        r#"/\b^[a]b\b/v.test('ab')"#,
        r#"/\b[a]b$\b/v.test('ab')"#,
        r#"/\b\b[a]b\b/v.test('ab')"#,
        r#"/\b[a]b\b\b/v.test('ab')"#,
        r#"/\b[\p{RGI_Emoji}]b\b/v.test('ab')"#,
        r#"/\b[]\p{RGI_Emoji}\b/v.test('')"#,
        r#"/\b[a]b\b/u.test('ab')"#,
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
        .eval(r#"var kept=/(?<whole>(\B[a]a\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"let m=kept.exec('aaa!');m.index===1&&m.groups.whole==='aa'&&m.indices[2][1]===3&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r#"var r=/\B[a]b\b/vg,sticky=/[a]a\B/vy,marker=0"#)
        .unwrap();
    // Each flat class succeeds and the suffix literal fails; cancellation is explicitly opted in.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval(r#"sticky.test(text)&&sticky.lastIndex===2"#),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn flat_v_set_class_literal_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full JSON and original spans are independently hand-derived finite expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>(\B[a]a\b))"#,
            JsString::from("aaa!"),
            "dvg",
            0,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\B[a]a\\b))"}"#,
        ),
        (
            r#"(?<whole>(\B[a]a\b))"#,
            JsString::from("aaa!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>(\B[a]a\b))"#,
            JsString::from("aaa!"),
            "dvy",
            1,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\B[a]a\\b))"}"#,
        ),
        (
            r#"(?<whole>(\b[\p{Any}]s\b))"#,
            JsString::from("Kſ!"),
            "divg",
            0,
            r#"{"matches":["Kſ","Kſ","Kſ"],"index":0,"input":"Kſ!","groups":{"whole":"Kſ"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\b[\\p{Any}]s\\b))"}"#,
        ),
        (
            r#"(?<whole>(\b[\P{Ll}]a\b))"#,
            JsString::from("Aa!"),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>(\b[\p{Any}]\u{1f601}\B))"#,
            JsString::from("A😀😁!"),
            "dvy",
            2,
            r#"{"matches":["😀😁","😀😁","😀😁"],"index":1,"input":"A😀😁!","groups":{"whole":"😀😁"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>(\\b[\\p{Any}]\\u{1f601}\\B))"}"#,
        ),
        (
            r#"(?<whole>(\b[\uD800]\uD800\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800!","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\b[\\uD800]\\uD800\\B))"}"#,
        ),
        (
            r#"^(?<whole>(\b[a]b\b))$"#,
            JsString::from("x\r\nab ac"),
            "dvmg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":3,"input":"x\r\nab ac","groups":{"whole":"ab"},"indices":[[3,5],[3,5],[3,5]],"indexGroups":{"whole":[3,5]},"lastIndex":5,"source":"^(?<whole>(\\b[a]b\\b))$"}"#,
        ),
        (
            r#"(?<whole>(\b[\]]\$\B))"#,
            JsString::from("x]$!"),
            "dvg",
            0,
            r#"{"matches":["]$","]$","]$"],"index":1,"input":"x]$!","groups":{"whole":"]$"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\b[\\]]\\$\\B))"}"#,
        ),
        (
            r#"^(?<whole>(\b[\u{61}]\u{62}\b))$"#,
            JsString::from("ab"),
            "dvg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":0,"input":"ab","groups":{"whole":"ab"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"^(?<whole>(\\b[\\u{61}]\\u{62}\\b))$"}"#,
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
fn literal_flat_v_set_state_context_and_consumers() {
    check(r#"/a[\q{b}]/v.test('ab')"#);
    check(r#"/\ba[b]\b/v.test('ab')"#);
    check(
        r#"let r=/\ba[b]\b/v,m=r.exec('ab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='ab'&&m.index===0&&m.input==='ab'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\ba[b]\\b'&&r.lastIndex===0&&r.exec('xb')===null&&r.exec('ac')===null&&r.exec('a')===null&&r.exec('xab')===null&&r.exec('abx')===null&&r.exec('!ab!').index===1"#,
    );
    check(
        r#"let r=/(?<whole>(a[b]))/dvg,m=r.exec('aab!');m!==null&&Array.isArray(m)&&m.length===3&&m[0]==='ab'&&m[1]==='ab'&&m[2]==='ab'&&m.index===1&&m.input==='aab!'&&m.groups.whole==='ab'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][0]===1&&m.indices[2][1]===3&&r.source==='(?<whole>(a[b]))'&&r.lastIndex===3&&r.exec('aab!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/a[b]/dvy;r.exec('aab!')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aab!')[0]==='ab')&&r.lastIndex===3&&r.exec('aab!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/s[K]/iv,m=r.exec('ſK!');m!==null&&m[0]==='ſK'&&m.index===0&&r.test('sk')&&!/s[K]/v.test('ſK')&&!/s[K]/v.test('sK')&&!r.test('aK')&&!r.test('sa')"#,
    );
    check(
        r#"!/a[]/v.test('aa!')&&!/a[]/v.test('')&&/a[^b]/v.test('aX!')&&!/a[^b]/v.test('ab!')&&/a[b]/v.test('abx')&&/a[b]/v.exec('xab').index===1"#,
    );
    check(
        r#"let r=/\u{1f600}[\p{Any}]/dvy;r.lastIndex=1;let a=r.exec('😀😁😀😁!'),b=r.exec('😀😁😀😁!');a.index===0&&a[0]==='😀😁'&&a.indices[0][1]===4&&b.index===4&&b[0]==='😀😁'&&r.lastIndex===8&&r.exec('😀😁😀😁!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\uD800[\uD800]/dv,m=r.exec('A\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800'&&m.indices[0][1]===3&&/\uD83D[\uDE00]/v.exec('😀')===null"#,
    );
    check(
        r#"/\$[\]]/v.exec('x$]!')[0]==='$]'&&/^\u{61}[\u{62}]$/v.test('ab')&&/^\u0061[\u0062]$/v.test('ab')&&!/^\$[\]]$/v.test('$]\n')"#,
    );
    check(
        r#"let r=/^(?<whole>(a[b]))$/dvmg,m=r.exec('x\r\nab\u2028ac');m.index===3&&m[0]==='ab'&&m.indices[2][0]===3&&m.indices[2][1]===5&&r.lastIndex===5&&r.exec('x\r\nab\u2028ac')===null&&r.lastIndex===0&&!/^a[b]$/v.test('ab\n')&&/^a[b]/v.test('ab!')&&/a[b]$/v.test('xab')"#,
    );
    check(
        r#"let r=/(?<whole>(a[b]))/dvg,a=[...'aab! aab!'.matchAll(r)];a.length===2&&a[0][0]==='ab'&&a[0][1]==='ab'&&a[0][2]==='ab'&&a[0].index===1&&a[0].groups.whole==='ab'&&a[0].indices.groups.whole===a[0].indices[1]&&a[1].index===6&&a[1].indices[2][1]===8&&r.lastIndex===0"#,
    );
    check(
        r#"'aab! aab!'.replace(/(?<whole>(a[b]))/vg,'<$1:$<whole>>')==='a<ab:ab>! a<ab:ab>!'&&'aab!'.split(/(a[b])/v).join('|')==='a|ab|!'&&'aab!'.match(/a[b]/v)[0]==='ab'"#,
    );
    check(
        r#"let seen='',out='aab! aab!'.replace(/(?<whole>(a[b]))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:ab;6:ab;'"#,
    );
    check(r#"let r=/a[b]/vg;r.lastIndex=4;'aab!'.search(r)===1&&r.lastIndex===4"#);
    check(
        r#"let trace='',r=/\u{1f600}[\p{Any}]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁!'}});trace==='si'&&m.index===0&&m[0]==='😀😁'&&r.lastIndex===4"#,
    );
    check(
        r#"let seen=0,r=/a[b]/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aab!');m.index===1&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/s[K]/iv,c=new RegExp(r,'dvy');c.source==='s[K]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('ſK')===null&&c.lastIndex===0&&r.exec('ſK')[0]==='ſK'"#,
    );
    check(
        r#"let a=/a[b]/vy,b=/a[b]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aab!')}catch(e){x=e instanceof TypeError}try{b.exec('aab!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let marker=0,caught=false;try{eval('marker=1; /\\u{110000}[]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0"#,
    );
    check(
        r#"let marker=0,result,caught=false;try{result=eval("marker=1; /a[b]/v.test('ab')")}catch{caught=true}finally{marker=3}!caught&&marker===3&&result&&!/a[b]/v.test('xb')&&!/a[b]/v.test('ac')&&!/a[b]/v.test('a')&&/a[[b]]/v.test('ab')&&/a[b--c]/v.test('ab')&&/[a]b/v.test('ab')"#,
    );
    check(
        r#"/s[\p{Any}]/iv.exec('ſK')[0]==='ſK'&&!/s[\p{Any}]/v.test('ſK')&&/s[\p{Any}]/iv.exec('ſ😁')[0]==='ſ😁'&&!/s[\p{Any}]/iv.test('ſ')"#,
    );
    check(r#"/a[b][c]/v.test('abc')"#);
    check(
        r#"let r=/a[b][c]/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='a[b][c]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.test('xabc')&&r.test('abcx')"#,
    );
    for source in [
        r#"/a\b[b]/v.test('ab')"#,
        r#"/a^[b]/v.test('ab')"#,
        r#"/a$[b]/v.test('ab')"#,
        r#"/ab[c]/v.test('abc')"#,
        r#"/xa[b]/v.test('xab')"#,
        r#"/a[b]c/v.test('abc')"#,
        r#"/a+[b]/v.test('ab')"#,
        r#"/a?[b]/v.test('ab')"#,
        r#"/a[b]+/v.test('ab')"#,
        r#"/a[b]?/v.test('ab')"#,
        r#"/(a)[b]/v.test('ab')"#,
        r#"/a([b])/v.test('ab')"#,
        r#"/(a[b])+/v.test('ab')"#,
        r#"/a[b]|c/v.test('ab')"#,
        r#"/a[\p{RGI_Emoji}]/v.test('')"#,
        r#"/\p{RGI_Emoji}[]/v.test('')"#,
        r#"/a[b]/u.test('ab')"#,
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
        .eval(r#"var kept=/(?<whole>(a[b]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"let m=kept.exec('aab!');m.index===1&&m.groups.whole==='ab'&&m.indices[2][1]===3&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r#"var r=/a[b]/vg,sticky=/a[a]/vy,marker=0"#)
        .unwrap();
    // Literal candidates succeed while the flat suffix class fails; cancellation is explicitly opted in.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval(r#"sticky.test(text)&&sticky.lastIndex===2"#),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn literal_flat_v_set_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full JSON and original UTF16 spans are independently hand-derived finite expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>(a[b]))"#,
            JsString::from("aab!"),
            "dvg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":1,"input":"aab!","groups":{"whole":"ab"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(a[b]))"}"#,
        ),
        (
            r#"(?<whole>(a[b]))"#,
            JsString::from("aab!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>(a[b]))"#,
            JsString::from("aab!"),
            "dvy",
            1,
            r#"{"matches":["ab","ab","ab"],"index":1,"input":"aab!","groups":{"whole":"ab"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(a[b]))"}"#,
        ),
        (
            r#"(?<whole>(s[K]))"#,
            JsString::from("ſK!"),
            "divg",
            0,
            r#"{"matches":["ſK","ſK","ſK"],"index":0,"input":"ſK!","groups":{"whole":"ſK"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(s[K]))"}"#,
        ),
        (
            r#"^(?<whole>(a[b]))$"#,
            JsString::from("x\r\nab ac"),
            "dvmg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":3,"input":"x\r\nab ac","groups":{"whole":"ab"},"indices":[[3,5],[3,5],[3,5]],"indexGroups":{"whole":[3,5]},"lastIndex":5,"source":"^(?<whole>(a[b]))$"}"#,
        ),
        (
            r#"(?<whole>(a[]))"#,
            JsString::from("aa!"),
            "dvg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>(a[^b]))"#,
            JsString::from("aX!"),
            "dvg",
            0,
            r#"{"matches":["aX","aX","aX"],"index":0,"input":"aX!","groups":{"whole":"aX"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(a[^b]))"}"#,
        ),
        (
            r#"(?<whole>(\u{1f600}[\p{Any}]))"#,
            JsString::from("A😀😁!"),
            "dvy",
            2,
            r#"{"matches":["😀😁","😀😁","😀😁"],"index":1,"input":"A😀😁!","groups":{"whole":"😀😁"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>(\\u{1f600}[\\p{Any}]))"}"#,
        ),
        (
            r#"(?<whole>(\uD800[\uD800]))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800!","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\uD800[\\uD800]))"}"#,
        ),
        (
            r#"^(?<whole>(\$[\]]))$"#,
            JsString::from("$]"),
            "dvg",
            0,
            r#"{"matches":["$]","$]","$]"],"index":0,"input":"$]","groups":{"whole":"$]"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"^(?<whole>(\\$[\\]]))$"}"#,
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
fn literal_flat_v_set_word_state_context_and_consumers() {
    check(r#"/\ba[\q{b}]\b/v.test('ab')"#);
    check(
        r#"let r=/(?<whole>(\Ba[a]\b))/dvg,m=r.exec('aaa!');m!==null&&Array.isArray(m)&&m.length===3&&m[0]==='aa'&&m[1]==='aa'&&m[2]==='aa'&&m.index===1&&m.input==='aaa!'&&m.groups.whole==='aa'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][0]===1&&m.indices[2][1]===3&&r.source==='(?<whole>(\\Ba[a]\\b))'&&r.lastIndex===3&&r.exec('aaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\Ba[a]\b/dvy;r.exec('aaa!')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aaa!')[0]==='aa')&&r.lastIndex===3&&r.exec('aaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/\bs[\p{Any}]\b/iv.exec('ſK!')[0]==='ſK'&&!/\bs[\p{Any}]\b/v.test('ſK!')&&/\bs[K]\b/iv.exec('ſK!')[0]==='ſK'&&!/\bs[K]\b/v.test('ſK!')&&!/\bs[K]\b/v.test('sK!')&&!/\bs[K]\b/iv.test('aK!')&&!/\bs[K]\b/iv.test('sa!')"#,
    );
    check(
        r#"!/\Ba[]\B/v.test('aa')&&!/\Ba[]\B/v.test('')&&/\ba[^b]\b/v.test('aX!')&&!/\ba[^b]\b/v.test('ab!')"#,
    );
    check(
        r#"let r=/\b\u{1f600}[\p{Any}]\B/dvy;r.lastIndex=2;let m=r.exec('A😀😁!');m!==null&&m.index===1&&m[0]==='😀😁'&&m.indices[0][0]===1&&m.indices[0][1]===5&&r.lastIndex===5&&r.exec('A😀😁!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\b\uD800[\uD800]\B/dv,m=r.exec('A\uD800\uD800!');m.index===1&&m[0]==='\uD800\uD800'&&m.indices[0][1]===3&&/\b\uD83D[\uDE00]\B/v.exec('A😀!')===null"#,
    );
    check(
        r#"/\b\$[\]]\B/v.exec('x$]!')[0]==='$]'&&/^\B\$[\]]\B$/v.test('$]')&&/^\b\u{61}[\u{62}]\b$/v.test('ab')&&/^\b\u0061[\u0062]\b$/v.test('ab')&&!/^\B\$[\]]\B$/v.test('$]\n')"#,
    );
    check(
        r#"let r=/^(?<whole>(\ba[b]\b))$/dvmg,m=r.exec('x\r\nab\u2028ac');m.index===3&&m[0]==='ab'&&m.indices[2][0]===3&&m.indices[2][1]===5&&r.lastIndex===5&&r.exec('x\r\nab\u2028ac')===null&&r.lastIndex===0&&!/^\ba[b]\b$/v.test('ab\n')&&/^\ba[b]\b/v.test('ab!')&&/\ba[b]\b$/v.test('!ab')"#,
    );
    check(
        r#"let r=/(?<whole>(\Ba[a]\b))/dvg,a=[...'aaa! aaa!'.matchAll(r)];a.length===2&&a[0][0]==='aa'&&a[0][1]==='aa'&&a[0][2]==='aa'&&a[0].index===1&&a[0].groups.whole==='aa'&&a[0].indices.groups.whole===a[0].indices[1]&&a[1].index===6&&a[1].indices[2][1]===8&&r.lastIndex===0"#,
    );
    check(
        r#"'aaa! aaa!'.replace(/(?<whole>(\Ba[a]\b))/vg,'<$1:$<whole>>')==='a<aa:aa>! a<aa:aa>!'&&'aaa!'.split(/(\Ba[a]\b)/v).join('|')==='a|aa|!'&&'aaa!'.match(/\Ba[a]\b/v)[0]==='aa'"#,
    );
    check(
        r#"let seen='',out='aaa! aaa!'.replace(/(?<whole>(\Ba[a]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return 'X'});out==='aX! aX!'&&seen==='1:aa;6:aa;'"#,
    );
    check(r#"let r=/\Ba[a]\b/vg;r.lastIndex=4;'aaa!'.search(r)===1&&r.lastIndex===4"#);
    check(
        r#"let trace='',r=/\b\u{1f600}[\p{Any}]\B/vy;r.lastIndex={valueOf(){trace+='i';return 2}};let m=r.exec({toString(){trace+='s';return 'A😀😁!'}});trace==='si'&&m.index===1&&m[0]==='😀😁'&&r.lastIndex===5"#,
    );
    check(
        r#"let seen=0,r=/\Ba[a]\b/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aaa!');m.index===1&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/\bs[K]\b/iv,c=new RegExp(r,'dvy');c.source==='\\bs[K]\\b'&&c.unicodeSets&&!c.ignoreCase&&c.exec('ſK')===null&&c.lastIndex===0&&r.exec('ſK')[0]==='ſK'"#,
    );
    check(
        r#"let a=/\Ba[a]\b/vy,b=/\Ba[a]\b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aaa!')}catch(e){x=e instanceof TypeError}try{b.exec('aaa!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let marker=0,caught=false;try{eval('marker=1; /\\b\\u{110000}[]\\b/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0"#,
    );
    check(
        r#"let marker=0,result,caught=false;try{result=eval("marker=1; /\\ba[b]\\b/v.test('ab')")}catch{caught=true}finally{marker=3}!caught&&marker===3&&result&&!/\ba[b]\b/v.test('xb')&&!/\ba[b]\b/v.test('ac')&&!/\ba[b]\b/v.test('a')&&!/\ba[b]\b/v.test('xab')&&!/\ba[b]\b/v.test('abx')"#,
    );
    check(
        r#"/\ba[b]/v.exec('abcx')[0]==='ab'&&/\ba[b]/v.exec('!ab').index===1&&!/\ba[b]/v.test('xab')&&/a[b]\b/v.exec('xab').index===1&&!/a[b]\b/v.test('abx')&&/\b[a]b\b/v.test('ab')&&/a[[b]]/v.test('ab')&&/a[b]/v.test('ab')&&/\ba[[b]]\b/v.test('ab')"#,
    );
    check(r#"/^\B\$[\p{Any}]\b$/iv.test('$K')&&!/^\B\$[\p{Any}]\b$/v.test('$K')"#);
    check(r#"/\ba[b][c]\b/v.test('abc')"#);
    check(
        r#"let r=/\ba[b][c]\b/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\ba[b][c]\\b'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&!r.test('xabc')&&!r.test('abcx')"#,
    );
    for source in [
        r#"/\ba[b]c\b/v.test('abc')"#,
        r#"/\bxa[b]\b/v.test('xab')"#,
        r#"/\b(a)[b]\b/v.test('ab')"#,
        r#"/\ba([b])\b/v.test('ab')"#,
        r#"/(\ba[b]\b)+/v.test('ab')"#,
        r#"/\ba+[b]\b/v.test('ab')"#,
        r#"/\ba[b]+\b/v.test('ab')"#,
        r#"/\ba[b]?\b/v.test('ab')"#,
        r#"/\ba[b]\b|c/v.test('ab')"#,
        r#"/\ba\b[b]\b/v.test('ab')"#,
        r#"/\b^a[b]\b/v.test('ab')"#,
        r#"/\ba[b]$\b/v.test('ab')"#,
        r#"/\b\ba[b]\b/v.test('ab')"#,
        r#"/\ba[b]\b\b/v.test('ab')"#,
        r#"/\ba[\p{RGI_Emoji}]\b/v.test('')"#,
        r#"/\b\p{RGI_Emoji}[]\b/v.test('')"#,
        r#"/\ba[b]\b/u.test('ab')"#,
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
        .eval(r#"var kept=/(?<whole>(\Ba[a]\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"let m=kept.exec('aaa!');m.index===1&&m.groups.whole==='aa'&&m.indices[2][1]===3&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r#"var r=/\Ba[b]\b/vg,sticky=/a[a]\B/vy,marker=0"#)
        .unwrap();
    // Literal candidates succeed while the suffix class fails; cancellation is explicitly opted in.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval(r#"sticky.test(text)&&sticky.lastIndex===2"#),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn literal_flat_v_set_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full JSON and original UTF16 spans are independently hand-derived finite expectations.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>(\Ba[a]\b))"#,
            JsString::from("aaa!"),
            "dvg",
            0,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\Ba[a]\\b))"}"#,
        ),
        (
            r#"(?<whole>(\Ba[a]\b))"#,
            JsString::from("aaa!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>(\Ba[a]\b))"#,
            JsString::from("aaa!"),
            "dvy",
            1,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\Ba[a]\\b))"}"#,
        ),
        (
            r#"(?<whole>(\bs[\p{Any}]\b))"#,
            JsString::from("ſK!"),
            "divg",
            0,
            r#"{"matches":["ſK","ſK","ſK"],"index":0,"input":"ſK!","groups":{"whole":"ſK"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\bs[\\p{Any}]\\b))"}"#,
        ),
        (
            r#"(?<whole>(\ba[]\b))"#,
            JsString::from("aa!"),
            "dvg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>(\ba[^b]\b))"#,
            JsString::from("aX!"),
            "dvg",
            0,
            r#"{"matches":["aX","aX","aX"],"index":0,"input":"aX!","groups":{"whole":"aX"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>(\\ba[^b]\\b))"}"#,
        ),
        (
            r#"(?<whole>(\b\u{1f600}[\p{Any}]\B))"#,
            JsString::from("A😀😁!"),
            "dvy",
            2,
            r#"{"matches":["😀😁","😀😁","😀😁"],"index":1,"input":"A😀😁!","groups":{"whole":"😀😁"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>(\\b\\u{1f600}[\\p{Any}]\\B))"}"#,
        ),
        (
            r#"(?<whole>(\b\uD800[\uD800]\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800!","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\b\\uD800[\\uD800]\\B))"}"#,
        ),
        (
            r#"^(?<whole>(\ba[b]\b))$"#,
            JsString::from("x\r\nab ac"),
            "dvmg",
            0,
            r#"{"matches":["ab","ab","ab"],"index":3,"input":"x\r\nab ac","groups":{"whole":"ab"},"indices":[[3,5],[3,5],[3,5]],"indexGroups":{"whole":[3,5]},"lastIndex":5,"source":"^(?<whole>(\\ba[b]\\b))$"}"#,
        ),
        (
            r#"^(?<whole>(\B\$[\]]\B))$"#,
            JsString::from("$]"),
            "dvg",
            0,
            r#"{"matches":["$]","$]","$]"],"index":0,"input":"$]","groups":{"whole":"$]"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"^(?<whole>(\\B\\$[\\]]\\B))$"}"#,
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
fn flat_v_set_literal_flat_v_set_state_context_and_consumers() {
    check(
        r#"let r=/(?<whole>([a]a[a]\b))/dvg,m=r.exec('aaaa!');m!==null&&Array.isArray(m)&&m.length===3&&m[0]==='aaa'&&m[1]==='aaa'&&m[2]==='aaa'&&m.index===1&&m.input==='aaaa!'&&m.groups.whole==='aaa'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][0]===1&&m.indices[2][1]===4&&r.source==='(?<whole>([a]a[a]\\b))'&&r.lastIndex===4&&r.exec('aaaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[a]a[a]\b/dvy;r.exec('aaaa!')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aaaa!')[0]==='aaa')&&r.lastIndex===4&&r.exec('aaaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\b[K]s[a]\b/div,m=r.exec('KſA!');m!==null&&m[0]==='KſA'&&m.index===0&&m.indices[0][1]===3&&!/\b[K]s[a]\b/v.test('KsA!')&&!/\b[K]s[a]\b/v.test('Kſa!')&&!/\b[K]s[a]\b/v.test('KsA!')&&!/\b[K]s[a]\b/iv.test('asA!')&&!/\b[K]s[a]\b/iv.test('Kxa!')&&!/\b[K]s[a]\b/iv.test('Ksb!')"#,
    );
    check(
        r#"let a=/^\B[\p{Any}]\$[\p{Any}]\b$/iv,b=/^\B[\p{Any}]\$[\p{Any}]\b$/v;a.test('!$K')&&a.test('!$ſ')&&!b.test('!$K')&&!b.test('!$ſ')&&a.exec('!$K')[0]==='!$K'"#,
    );
    check(
        r#"/[]b[c]/v.exec('abc')===null&&/[a]b[]/v.exec('abc')===null&&/[a]b[^c]/v.exec('abx!')[0]==='abx'&&!/[a]b[^c]/v.test('abc!')"#,
    );
    check(
        r#"/[\P{Ll}]b[c]/v.test('Abc')&&!/[\P{Ll}]b[c]/iv.test('Abc')&&!/[\P{Ll}]b[c]/iv.test('abc')&&/[\P{Ll}]b[c]/iv.test('!bc')"#,
    );
    check(
        r#"let r=/\b[\u{1f600}]\u{1f601}[\u{1f600}]\B/dvy;r.lastIndex=2;let m=r.exec('A😀😁😀!');m!==null&&m.index===1&&m[0]==='😀😁😀'&&m.indices[0][0]===1&&m.indices[0][1]===7&&r.lastIndex===7&&r.exec('A😀😁😀!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\b[\uD800]\uD800[\uD800]\B/dv,m=r.exec('A\uD800\uD800\uD800!');m!==null&&m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&/[\uD83D]\uDE00[a]/v.exec('😀a')===null"#,
    );
    check(
        r#"/^\B[\]]\$[!]\B$/v.test(']$!')&&!/^\B[\]]\$[!]\B$/v.test(']$!\n')&&/^[\u{61}]\u{62}[\u{63}]$/v.test('abc')&&/^[\u0061]\u0062[\u0063]$/v.test('abc')"#,
    );
    check(
        r#"let r=/^(?<whole>(\b[a]b[c]\b))$/dvmg,m=r.exec('x\r\nabc\u2028abx');m!==null&&m.index===3&&m[0]==='abc'&&m.indices[2][0]===3&&m.indices[2][1]===6&&r.lastIndex===6&&r.exec('x\r\nabc\u2028abx')===null&&r.lastIndex===0&&!/^\b[a]b[c]\b$/v.test('abc\n')"#,
    );
    check(
        r#"/\b[a]b[c]/v.exec('abcx')[0]==='abc'&&/\b[a]b[c]/v.exec('abcx').index===0&&/[a]b[c]\b/v.exec('xabc').index===1&&!/\b[a]b[c]/v.test('xabc')&&!/[a]b[c]\b/v.test('abcx')&&/[[a]]b[[c]]/v.test('abc')&&/\b[[a]]b[[c]]\b/v.test('abc')&&/\ba[b]\b/v.test('ab')&&/\b[a]b\b/v.test('ab')&&/[a][b][c]/v.test('abc')"#,
    );
    check(
        r#"let r=/(?<whole>([a]b[c]\b))/dvg,a=[...'😀abc aabc'.matchAll(r)];a.length===2&&a[0][0]==='abc'&&a[0][1]==='abc'&&a[0][2]==='abc'&&a[0].index===2&&a[0].groups.whole==='abc'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===5&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0"#,
    );
    check(
        r#"'😀abc aabc'.replace(/(?<whole>([a]b[c]\b))/vg,'<$1:$<whole>>')==='😀<abc:abc> a<abc:abc>'&&'😀abcx'.split(/([a]b[c])/v).join('|')==='😀|abc|x'&&'aabc'.match(/[a]b[c]/v)[0]==='abc'"#,
    );
    check(
        r#"let seen='',out='😀abc aabc'.replace(/(?<whole>([a]b[c]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return '!'});out==='😀! a!'&&seen==='2:abc;7:abc;'"#,
    );
    check(r#"let r=/[a]b[c]/vg;r.lastIndex=1;'😀aabc'.search(r)===3&&r.lastIndex===1"#);
    check(
        r#"let trace='',r=/[\p{Any}]\$[\p{Any}]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀$K'}});trace==='si'&&m.index===0&&m[0]==='😀$K'&&r.lastIndex===4"#,
    );
    check(
        r#"let seen=0,r=/[a]b[c]/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aabc');m.index===1&&m[0]==='abc'&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[K]s[a]/iv,c=new RegExp(r,'dvy');c.source==='[K]s[a]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſA')===null&&c.lastIndex===0&&r.exec('KſA')[0]==='KſA'"#,
    );
    check(
        r#"let a=/[a]b[c]/vy,b=/[a]b[c]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aabc')}catch(e){x=e instanceof TypeError}try{b.exec('xabc')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let marker=0,caught=false;try{eval('marker=1; /[a&&]b[c]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0"#,
    );
    check(r#"/[[a]]b[c]/v.test('abc')"#);
    check(
        r#"let r=/[[a]]b[c]/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]]b[c]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc').index===1"#,
    );
    check(r#"/[a]b[[c]]/v.test('abc')"#);
    check(
        r#"let r=/[a]b[[c]]/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a]b[[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc').index===1"#,
    );
    for source in [
        r#"/[a]bc[c]/v.test('abcc')"#,
        r#"/x[a]b[c]/v.test('xabc')"#,
        r#"/[a]b[c]x/v.test('abcx')"#,
        r#"/[a]b[c][d]/v.test('abcd')"#,
        r#"/([a])b[c]/v.test('abc')"#,
        r#"/[a](b)[c]/v.test('abc')"#,
        r#"/[a]b([c])/v.test('abc')"#,
        r#"/(?:[a]b[c])+/v.test('abc')"#,
        r#"/[a]+b[c]/v.test('abc')"#,
        r#"/[a]b[c]?/v.test('abc')"#,
        r#"/[a]\bb[c]/v.test('abc')"#,
        r#"/\b^[a]b[c]/v.test('abc')"#,
        r#"/[a]b[c]$\b/v.test('abc')"#,
        r#"/\b\b[a]b[c]/v.test('abc')"#,
        r#"/[a]b[c]\b\b/v.test('abc')"#,
        r#"/[a]b[\q{c}]/v.test('abc')"#,
        r#"/[]b[\p{RGI_Emoji}]/v.test('')"#,
        r#"/[\q{a}]b[]/v.test('')"#,
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
        .eval(r#"var kept=/(?<whole>([a]a[a]\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([a]a[a]\\b))'&&kept.lastIndex===1&&kept.exec('aaaa!').groups.whole==='aaa'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[a]a[c]/vg,marker=0"#).unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn flat_v_set_literal_flat_v_set_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Ten full JSON payloads and original UTF16 spans were hand-derived before Runtime source.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([a]a[a]\b))"#,
            JsString::from("aaaa!"),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a]a[a]\\b))"}"#,
        ),
        (
            r#"(?<whole>([a]a[a]\b))"#,
            JsString::from("aaaa!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([a]a[a]\b))"#,
            JsString::from("aaaa!"),
            "dvy",
            1,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a]a[a]\\b))"}"#,
        ),
        (
            r#"^(?<whole>(\B[\p{Any}]\$[\p{Any}]\b))$"#,
            JsString::from("!$K"),
            "divg",
            0,
            r#"{"matches":["!$K","!$K","!$K"],"index":0,"input":"!$K","groups":{"whole":"!$K"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"^(?<whole>(\\B[\\p{Any}]\\$[\\p{Any}]\\b))$"}"#,
        ),
        (
            r#"(?<whole>([]b[c]))"#,
            JsString::from("abc!"),
            "dvg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([a]b[^c]))"#,
            JsString::from("abx!"),
            "dvg",
            0,
            r#"{"matches":["abx","abx","abx"],"index":0,"input":"abx!","groups":{"whole":"abx"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>([a]b[^c]))"}"#,
        ),
        (
            r#"(?<whole>(\b[\u{1f600}]\u{1f601}[\u{1f600}]\B))"#,
            JsString::from("A😀😁😀!"),
            "dvy",
            2,
            r#"{"matches":["😀😁😀","😀😁😀","😀😁😀"],"index":1,"input":"A😀😁😀!","groups":{"whole":"😀😁😀"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\b[\\u{1f600}]\\u{1f601}[\\u{1f600}]\\B))"}"#,
        ),
        (
            r#"(?<whole>(\b[\uD800]\uD800[\uD800]\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b[\\uD800]\\uD800[\\uD800]\\B))"}"#,
        ),
        (
            r#"^(?<whole>(\b[a]b[c]\b))$"#,
            JsString::from("x\r\nabc\u{2028}abx"),
            "dvmg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":3,"input":"x\r\nabc abx","groups":{"whole":"abc"},"indices":[[3,6],[3,6],[3,6]],"indexGroups":{"whole":[3,6]},"lastIndex":6,"source":"^(?<whole>(\\b[a]b[c]\\b))$"}"#,
        ),
        (
            r#"(?<whole>(\b[a]b[c]))"#,
            JsString::from("abcx"),
            "dvg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":0,"input":"abcx","groups":{"whole":"abc"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\b[a]b[c]))"}"#,
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
fn literal_flat_v_set_pair_state_context_and_consumers() {
    check(
        r#"let r=/(?<whole>(a[b][c]))/dvg,m=r.exec('aabc');m!==null&&Array.isArray(m)&&m.length===3&&m[0]==='abc'&&m[1]==='abc'&&m[2]==='abc'&&m.index===1&&m.input==='aabc'&&m.groups.whole==='abc'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][0]===1&&m.indices[2][1]===4&&r.source==='(?<whole>(a[b][c]))'&&r.lastIndex===4&&r.exec('aabc')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/a[a][b]/dvg,m=r.exec('aaab!');m!==null&&m.index===1&&m[0]==='aab'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/a[a][a]\b/dvg,m=r.exec('aaaa!');m!==null&&m.index===1&&m[0]==='aaa'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/a[a][a]\b/dvy;r.exec('aaaa!')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aaaa!')[0]==='aaa')&&r.lastIndex===4&&r.exec('aaaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/\bc[a][b]/v.test('cab')&&/\Bc[a][b]/v.test('xcab')&&/c[a][b]\b/v.test('cab!')&&/c[a][b]\B/v.test('cabx')&&/\bc[a][b]\b/v.test('!cab!')&&/\bc[a][b]\B/v.test('!cabx')&&/\Bc[a][b]\b/v.test('xcab!')&&/\Bc[a][b]\B/v.test('xcabx')&&!/\bc[a][b]/v.test('xcab')&&!/c[a][b]\b/v.test('cabx')&&!/\Bc[a][b]/v.test('!cab')&&!/c[a][b]\B/v.test('cab!')"#,
    );
    check(
        r#"/\bk[s][A]\b/iv.test('Kſa!')&&!/\bk[s][A]\b/v.test('KsA!')&&!/\bk[s][A]\b/v.test('kſA!')&&!/\bk[s][A]\b/v.test('ksa!')&&/\bk[s][A]\b/v.test('ksA!')&&!/\bk[s][A]\b/iv.test('asA!')&&!/\bk[s][A]\b/iv.test('kxa!')&&!/\bk[s][A]\b/iv.test('ksb!')"#,
    );
    check(
        r#"let a=/^\B\$[\p{Any}][\p{Any}]\b$/iv,b=/^\B\$[\p{Any}][\p{Any}]\b$/v;a.test('$!K')&&a.test('$!ſ')&&!b.test('$!K')&&!b.test('$!ſ')&&a.exec('$!K')[0]==='$!K'"#,
    );
    check(
        r#"/a[][\p{Any}]/v.exec('abc')===null&&/a[b][]/v.exec('abc')===null&&/a[^b][c]/v.exec('axc!')[0]==='axc'&&!/a[^b][c]/v.test('abc!')&&/a[b][^c]/v.exec('abx!')[0]==='abx'&&!/a[b][^c]/v.test('abc!')"#,
    );
    check(
        r#"/x[\P{Ll}][s]/v.test('xAs')&&!/x[\P{Ll}][s]/iv.test('xAs')&&!/x[\P{Ll}][s]/iv.test('xas')&&/x[\P{Ll}][s]/iv.test('x!s')"#,
    );
    check(
        r#"let r=/\b\u{1f600}[\u{1f601}][\u{1f600}]\B/dvy;r.lastIndex=2;let m=r.exec('A😀😁😀!');m!==null&&m.index===1&&m[0]==='😀😁😀'&&m.indices[0][0]===1&&m.indices[0][1]===7&&r.lastIndex===7&&r.exec('A😀😁😀!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\b\uD800[\uD800][\uD800]\B/dv,m=r.exec('A\uD800\uD800\uD800!');m!==null&&m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&/\uD83D[\uDE00][a]/v.exec('😀a')===null&&/a[\uD83D][\uDE00]/v.exec('a😀')===null"#,
    );
    check(
        r#"/^\B\$[\]][!]\B$/v.test('$]!')&&!/^\B\$[\]][!]\B$/v.test('$]!\n')&&/^\u{61}[\u{62}][\u{63}]$/v.test('abc')&&/^\u0061[\u0062][\u0063]$/v.test('abc')"#,
    );
    check(
        r#"let r=/^(?<whole>(\bc[a][b]\b))$/dvmg,m=r.exec('x\r\ncab\u2028cax');m!==null&&m.index===3&&m[0]==='cab'&&m.indices[2][0]===3&&m.indices[2][1]===6&&r.lastIndex===6&&r.exec('x\r\ncab\u2028cax')===null&&r.lastIndex===0&&!/^\bc[a][b]\b$/v.test('cab\n')"#,
    );
    check(
        r#"/\bc[a][b]/v.exec('cabx')[0]==='cab'&&/\bc[a][b]/v.exec('cabx').index===0&&/c[a][b]\b/v.exec('xcab').index===1&&/c[[a]][[b]]/v.test('cab')&&/\bc[[a]][[b]]\b/v.test('cab')&&/[a]b[c]/v.test('abc')&&/\ba[b]\b/v.test('ab')&&/\b[a]b\b/v.test('ab')&&/[a][b][c]/v.test('abc')"#,
    );
    check(
        r#"let r=/(?<whole>(a[b][c]\b))/dvg,a=[...'😀abc aabc'.matchAll(r)];a.length===2&&a[0][0]==='abc'&&a[0][1]==='abc'&&a[0][2]==='abc'&&a[0].index===2&&a[0].groups.whole==='abc'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===5&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0"#,
    );
    check(
        r#"'😀abc aabc'.replace(/(?<whole>(a[b][c]\b))/vg,'<$1:$<whole>>')==='😀<abc:abc> a<abc:abc>'&&'😀abcx'.split(/(a[b][c])/v).join('|')==='😀|abc|x'&&'aabc'.match(/a[b][c]/v)[0]==='abc'"#,
    );
    check(
        r#"let seen='',out='😀abc aabc'.replace(/(?<whole>(a[b][c]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return '!'});out==='😀! a!'&&seen==='2:abc;7:abc;'"#,
    );
    check(r#"let r=/a[b][c]/vg;r.lastIndex=1;'😀aabc'.search(r)===3&&r.lastIndex===1"#);
    check(
        r#"let trace='',r=/\u{1f600}[\p{Any}][a]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5"#,
    );
    check(
        r#"let seen=0,r=/a[b][c]/v;r.lastIndex={valueOf(){seen++;return 1}};let m=r.exec('aabc');m.index===1&&m[0]==='abc'&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/k[s][A]/iv,c=new RegExp(r,'dvy');c.source==='k[s][A]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('Kſa')===null&&c.lastIndex===0&&r.exec('Kſa')[0]==='Kſa'"#,
    );
    check(
        r#"let a=/a[b][c]/vy,b=/a[b][c]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aabc')}catch(e){x=e instanceof TypeError}try{b.exec('xabc')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let marker=0,caught=false;try{eval('marker=1; /a[b&&][c]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0"#,
    );
    check(r#"/c[[a]][b]/v.test('cab')"#);
    check(
        r#"let r=/c[[a]][b]/v,m=r.exec('cab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[[a]][b]'&&r.lastIndex===0&&!r.test('xab')&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&r.test('cabx')&&r.exec('xcab').index===1"#,
    );
    check(r#"/\bc[[a]][b]/v.test('cab')"#);
    check(
        r#"let r=/\bc[[a]][b]/v,m=r.exec('cab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\bc[[a]][b]'&&r.lastIndex===0&&!r.test('xab')&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&r.test('cabx')&&!r.test('xcab')&&r.exec('!cab').index===1"#,
    );
    check(r#"/c[a--b][c]/v.test('cac')"#);
    check(
        r#"let r=/c[a--b][c]/v,m=r.exec('cac');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cac'&&m.index===0&&m.input==='cac'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[a--b][c]'&&r.lastIndex===0&&!r.test('xac')&&!r.test('cxc')&&!r.test('cax')&&!r.test('ca')&&r.test('cacx')&&r.exec('xcac').index===1"#,
    );
    check(r#"/c[a][[b]]/v.test('cab')"#);
    check(
        r#"let r=/c[a][[b]]/v,m=r.exec('cab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[a][[b]]'&&r.lastIndex===0&&!r.test('xab')&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&r.test('cabx')&&r.exec('xcab').index===1"#,
    );
    check(r#"/\bc[a][[b]]/v.test('cab')"#);
    check(
        r#"let r=/\bc[a][[b]]/v,m=r.exec('cab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\bc[a][[b]]'&&r.lastIndex===0&&!r.test('xab')&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&r.test('cabx')&&!r.test('xcab')&&r.exec('!cab').index===1"#,
    );
    check(r#"/c[a][c--b]/v.test('cac')"#);
    check(
        r#"let r=/c[a][c--b]/v,m=r.exec('cac');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cac'&&m.index===0&&m.input==='cac'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[a][c--b]'&&r.lastIndex===0&&!r.test('xac')&&!r.test('cxc')&&!r.test('cax')&&!r.test('ca')&&r.test('cacx')&&r.exec('xcac').index===1"#,
    );
    for source in [
        r#"/ca[a][b]/v.test('caab')"#,
        r#"/c[a][b]d/v.test('cabd')"#,
        r#"/c[a][b][c]/v.test('cabc')"#,
        r#"/(c)[a][b]/v.test('cab')"#,
        r#"/c([a])[b]/v.test('cab')"#,
        r#"/c[a]([b])/v.test('cab')"#,
        r#"/(?:c[a][b])+/v.test('cab')"#,
        r#"/c+[a][b]/v.test('cab')"#,
        r#"/c[a]+[b]/v.test('cab')"#,
        r#"/c[a][b]?/v.test('cab')"#,
        r#"/c\b[a][b]/v.test('cab')"#,
        r#"/\b^c[a][b]/v.test('cab')"#,
        r#"/c[][\p{RGI_Emoji}]/v.test('')"#,
        r#"/c[][\q{b}]/v.test('')"#,
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
        .eval(r#"var kept=/(?<whole>(a[a][a]\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>(a[a][a]\\b))'&&kept.lastIndex===1&&kept.exec('aaaa!').groups.whole==='aaa'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/a[a][c]/vg,marker=0"#).unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn literal_flat_v_set_pair_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full expected JSON and original UTF16 spans were hand-derived before Runtime source.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>(a[b][c]))"#,
            JsString::from("aabc"),
            "dvg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":1,"input":"aabc","groups":{"whole":"abc"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(a[b][c]))"}"#,
        ),
        (
            r#"(?<whole>(a[a][b]))"#,
            JsString::from("aaab!"),
            "dvg",
            0,
            r#"{"matches":["aab","aab","aab"],"index":1,"input":"aaab!","groups":{"whole":"aab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(a[a][b]))"}"#,
        ),
        (
            r#"(?<whole>(a[a][a]\b))"#,
            JsString::from("aaaa!"),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(a[a][a]\\b))"}"#,
        ),
        (
            r#"(?<whole>(a[a][a]\b))"#,
            JsString::from("aaaa!"),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>(a[a][a]\b))"#,
            JsString::from("aaaa!"),
            "dvy",
            1,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(a[a][a]\\b))"}"#,
        ),
        (
            r#"(?<whole>(\bc[a][b]))"#,
            JsString::from("cabx"),
            "dvg",
            0,
            r#"{"matches":["cab","cab","cab"],"index":0,"input":"cabx","groups":{"whole":"cab"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\bc[a][b]))"}"#,
        ),
        (
            r#"(?<whole>(\b\u{1f600}[\u{1f601}][\u{1f600}]\B))"#,
            JsString::from("A😀😁😀!"),
            "dvy",
            2,
            r#"{"matches":["😀😁😀","😀😁😀","😀😁😀"],"index":1,"input":"A😀😁😀!","groups":{"whole":"😀😁😀"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\b\\u{1f600}[\\u{1f601}][\\u{1f600}]\\B))"}"#,
        ),
        (
            r#"(?<whole>(\b\uD800[\uD800][\uD800]\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b\\uD800[\\uD800][\\uD800]\\B))"}"#,
        ),
        (
            r#"^(?<whole>(\B\$[\p{Any}][\p{Any}]\b))$"#,
            JsString::from("$!K"),
            "divg",
            0,
            r#"{"matches":["$!K","$!K","$!K"],"index":0,"input":"$!K","groups":{"whole":"$!K"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"^(?<whole>(\\B\\$[\\p{Any}][\\p{Any}]\\b))$"}"#,
        ),
        (
            r#"(?<whole>(a[][\p{Any}]))"#,
            JsString::from("abc!"),
            "dvg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
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
fn flat_v_set_pair_literal_state_context_and_consumers() {
    check(
        r#"let r=/(?<whole>([a][b]c))/dvg,m=r.exec('aabc');m!==null&&Array.isArray(m)&&m.length===3&&m[0]==='abc'&&m[1]==='abc'&&m[2]==='abc'&&m.index===1&&m.input==='aabc'&&m.groups.whole==='abc'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][0]===1&&m.indices[2][1]===4&&r.source==='(?<whole>([a][b]c))'&&r.lastIndex===4&&r.exec('aabc')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[a][a]b/dvg,m=r.exec('aaab!');m!==null&&m.index===1&&m[0]==='aab'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/[a][a]a\b/dvg,m=r.exec('aaaa!');m!==null&&m.index===1&&m[0]==='aaa'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/[a][a]a\b/dvy;r.exec('aaaa!')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aaaa!')[0]==='aaa')&&r.lastIndex===4&&r.exec('aaaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/\b[a][b]c/v.test('abc')&&/\B[a][b]c/v.test('xabc')&&/[a][b]c\b/v.test('abc!')&&/[a][b]c\B/v.test('abcx')&&/\b[a][b]c\b/v.test('!abc!')&&/\b[a][b]c\B/v.test('!abcx')&&/\B[a][b]c\b/v.test('xabc!')&&/\B[a][b]c\B/v.test('xabcx')&&!/\b[a][b]c/v.test('xabc')&&!/[a][b]c\b/v.test('abcx')&&!/\B[a][b]c/v.test('!abc')&&!/[a][b]c\B/v.test('abc!')"#,
    );
    check(
        r#"/\b[K][s]a\b/iv.test('KſA!')&&!/\b[K][s]a\b/v.test('Ksa!')&&!/\b[K][s]a\b/v.test('Kſa!')&&!/\b[K][s]a\b/v.test('KsA!')&&/\b[K][s]a\b/v.test('Ksa!')&&!/\b[K][s]a\b/iv.test('asa!')&&!/\b[K][s]a\b/iv.test('Kxa!')&&!/\b[K][s]a\b/iv.test('Ksb!')"#,
    );
    check(
        r#"let a=/^\b[\p{Any}][\p{Any}]\$\B$/iv,b=/^\b[\p{Any}][\p{Any}]\$\B$/v;a.test('K!$')&&a.test('ſ!$')&&!b.test('K!$')&&!b.test('ſ!$')&&a.exec('K!$')[0]==='K!$'"#,
    );
    check(
        r#"/[][\p{Any}]c/v.exec('abc')===null&&/[a][]c/v.exec('abc')===null&&/[^a][b]c/v.exec('xbc!')[0]==='xbc'&&!/[^a][b]c/v.test('abc!')&&/[a][^b]c/v.exec('axc!')[0]==='axc'&&!/[a][^b]c/v.test('abc!')"#,
    );
    check(
        r#"/[\P{Ll}][s]x/v.test('Asx')&&!/[\P{Ll}][s]x/iv.test('Asx')&&!/[\P{Ll}][s]x/iv.test('asx')&&/[\P{Ll}][s]x/iv.test('!sx')"#,
    );
    check(
        r#"let r=/\b[\u{1f600}][\u{1f601}]\u{1f600}\B/dvy;r.lastIndex=2;let m=r.exec('A😀😁😀!');m!==null&&m.index===1&&m[0]==='😀😁😀'&&m.indices[0][0]===1&&m.indices[0][1]===7&&r.lastIndex===7&&r.exec('A😀😁😀!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\b[\uD800][\uD800]\uD800\B/dv,m=r.exec('A\uD800\uD800\uD800!');m!==null&&m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&/[\uD83D][\uDE00]a/v.exec('😀a')===null&&/[a][\uD83D]\uDE00/v.exec('a😀')===null"#,
    );
    check(
        r#"/^\b[a][\]]\$\B$/v.test('a]$')&&!/^\b[a][\]]\$\B$/v.test('a]$\n')&&/^[\u{61}][\u{62}]\u{63}$/v.test('abc')&&/^[\u0061][\u0062]\u0063$/v.test('abc')"#,
    );
    check(
        r#"let r=/^(?<whole>(\b[a][b]c\b))$/dvmg,m=r.exec('x\r\nabc\u2028axc');m!==null&&m.index===3&&m[0]==='abc'&&m.indices[2][0]===3&&m.indices[2][1]===6&&r.lastIndex===6&&r.exec('x\r\nabc\u2028axc')===null&&r.lastIndex===0&&!/^\b[a][b]c\b$/v.test('abc\n')"#,
    );
    check(
        r#"/\b[a][b]c/v.exec('abcx')[0]==='abc'&&/\b[a][b]c/v.exec('abcx').index===0&&/[a][b]c\b/v.exec('xabc').index===1&&/[[a]][[b]]c/v.test('abc')&&/\b[[a]][[b]]c\b/v.test('abc')&&/[a]b[c]/v.test('abc')&&/\ba[b]\b/v.test('ab')&&/\b[a]b\b/v.test('ab')&&/[a][b][c]/v.test('abc')"#,
    );
    check(
        r#"let r=/(?<whole>([a][b]c\b))/dvg,a=[...'😀abc aabc'.matchAll(r)];a.length===2&&a[0][0]==='abc'&&a[0][1]==='abc'&&a[0][2]==='abc'&&a[0].index===2&&a[0].groups.whole==='abc'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===5&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0"#,
    );
    check(
        r#"'😀abc aabc'.replace(/(?<whole>([a][b]c\b))/vg,'<$1:$<whole>>')==='😀<abc:abc> a<abc:abc>'&&'😀abcx'.split(/([a][b]c)/v).join('|')==='😀|abc|x'&&'aabc'.match(/[a][b]c/v)[0]==='abc'"#,
    );
    check(
        r#"let seen='',out='😀abc aabc'.replace(/(?<whole>([a][b]c\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return '!'});out==='😀! a!'&&seen==='2:abc;7:abc;'"#,
    );
    check(r#"let r=/[a][b]c/vg;r.lastIndex=1;'😀aabc'.search(r)===3&&r.lastIndex===1"#);
    check(
        r#"let trace='',r=/[\p{Any}][\u{1f601}]a/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5"#,
    );
    check(
        r#"let seen=0,r=/[a][b]c/v;r.lastIndex={valueOf(){seen++;return 3}};let m=r.exec('abcabc');m.index===0&&m[0]==='abc'&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[K][s]a/iv,c=new RegExp(r,'dvy');c.source==='[K][s]a'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſA')===null&&c.lastIndex===0&&r.exec('KſA')[0]==='KſA'"#,
    );
    check(
        r#"let a=/[a][b]c/vy,b=/[a][b]c/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aabc')}catch(e){x=e instanceof TypeError}try{b.exec('xabc')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let marker=0,caught=false;try{eval('marker=1; /[a][b&&]c/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0"#,
    );
    check(r#"/[[a]][b]c/v.test('abc')"#);
    check(
        r#"let r=/[[a]][b]c/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][b]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc').index===1"#,
    );
    check(r#"/\b[[a]][b]c/v.test('abc')"#);
    check(
        r#"let r=/\b[[a]][b]c/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[[a]][b]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&!r.test('xabc')&&r.exec('!abc').index===1"#,
    );
    check(r#"/[a--b][c]d/v.test('acd')"#);
    check(
        r#"let r=/[a--b][c]d/v,m=r.exec('acd');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='acd'&&m.index===0&&m.input==='acd'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a--b][c]d'&&r.lastIndex===0&&!r.test('xcd')&&!r.test('axd')&&!r.test('acx')&&!r.test('ac')&&r.exec('acdx')[0]==='acd'&&r.exec('acdx').index===0&&r.exec('xacd').index===1"#,
    );
    check(r#"/[a][[b]]c/v.test('abc')"#);
    check(
        r#"let r=/[a][[b]]c/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a][[b]]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc').index===1"#,
    );
    check(r#"/\b[a][[b]]c/v.test('abc')"#);
    check(
        r#"let r=/\b[a][[b]]c/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\b[a][[b]]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&!r.test('xabc')&&r.exec('!abc').index===1"#,
    );
    check(r#"/[a][c--b]d/v.test('acd')"#);
    check(
        r#"let r=/[a][c--b]d/v,m=r.exec('acd');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='acd'&&m.index===0&&m.input==='acd'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a][c--b]d'&&r.lastIndex===0&&!r.test('xcd')&&!r.test('axd')&&!r.test('acx')&&!r.test('ac')&&r.exec('acdx')[0]==='acd'&&r.exec('acdx').index===0&&r.exec('xacd').index===1"#,
    );
    for source in [
        r#"/[a][b]cd/v.test('abcd')"#,
        r#"/[a][b][c]d/v.test('abcd')"#,
        r#"/[a][b]c[d]/v.test('abcd')"#,
        r#"/([a])[b]c/v.test('abc')"#,
        r#"/[a]([b])c/v.test('abc')"#,
        r#"/[a][b](c)/v.test('abc')"#,
        r#"/(?:[a][b]c)+/v.test('abc')"#,
        r#"/[a]+[b]c/v.test('abc')"#,
        r#"/[a][b]?c/v.test('abc')"#,
        r#"/[a][b]c+/v.test('abc')"#,
        r#"/[a]\b[b]c/v.test('abc')"#,
        r#"/\b^[a][b]c/v.test('abc')"#,
        r#"/[][\p{RGI_Emoji}]c/v.test('')"#,
        r#"/[][\q{b}]c/v.test('')"#,
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
        .eval(r#"var kept=/(?<whole>([a][a]a\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([a][a]a\\b))'&&kept.lastIndex===1&&kept.exec('aaaa!').groups.whole==='aaa'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[a][a]c/vg,marker=0"#).unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn flat_v_set_pair_literal_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full expected JSON and original UTF16 spans were hand-derived before Runtime source.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([a][b]c))"#,
            JsString::from_code_units(vec![97, 97, 98, 99]),
            "dvg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":1,"input":"aabc","groups":{"whole":"abc"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a][b]c))"}"#,
        ),
        (
            r#"(?<whole>([a][a]b))"#,
            JsString::from_code_units(vec![97, 97, 97, 98, 33]),
            "dvg",
            0,
            r#"{"matches":["aab","aab","aab"],"index":1,"input":"aaab!","groups":{"whole":"aab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a][a]b))"}"#,
        ),
        (
            r#"(?<whole>([a][a]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a][a]a\\b))"}"#,
        ),
        (
            r#"(?<whole>([a][a]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([a][a]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            1,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a][a]a\\b))"}"#,
        ),
        (
            r#"(?<whole>(\b[a][b]c))"#,
            JsString::from_code_units(vec![97, 98, 99, 120]),
            "dvg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":0,"input":"abcx","groups":{"whole":"abc"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\b[a][b]c))"}"#,
        ),
        (
            r#"(?<whole>(\b[\u{1f600}][\u{1f601}]\u{1f600}\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56833, 55357, 56832, 33]),
            "dvy",
            2,
            r#"{"matches":["😀😁😀","😀😁😀","😀😁😀"],"index":1,"input":"A😀😁😀!","groups":{"whole":"😀😁😀"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\b[\\u{1f600}][\\u{1f601}]\\u{1f600}\\B))"}"#,
        ),
        (
            r#"(?<whole>(\b[\uD800][\uD800]\uD800\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b[\\uD800][\\uD800]\\uD800\\B))"}"#,
        ),
        (
            r#"^(?<whole>(\b[\p{Any}][\p{Any}]\$\B))$"#,
            JsString::from_code_units(vec![8490, 33, 36]),
            "divg",
            0,
            r#"{"matches":["K!$","K!$","K!$"],"index":0,"input":"K!$","groups":{"whole":"K!$"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"^(?<whole>(\\b[\\p{Any}][\\p{Any}]\\$\\B))$"}"#,
        ),
        (
            r#"(?<whole>([][\p{Any}]c))"#,
            JsString::from_code_units(vec![97, 98, 99, 33]),
            "dvg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
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
fn literal_checked_flat_v_set_pair_state_context_and_consumers() {
    check(
        r#"let r=/(?<whole>(c[[a]][b]))/dvg,m=r.exec('ccab');m!==null&&Array.isArray(m)&&m.length===3&&m[0]==='cab'&&m[1]==='cab'&&m[2]==='cab'&&m.index===1&&m.input==='ccab'&&m.groups.whole==='cab'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][0]===1&&m.indices[2][1]===4&&r.source==='(?<whole>(c[[a]][b]))'&&r.lastIndex===4&&r.exec('ccab')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/a[[a]][b]/dvg,m=r.exec('aaab!');m!==null&&m.index===1&&m[0]==='aab'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/a[[a]][a]\b/dvg,m=r.exec('aaaa!');m!==null&&m.index===1&&m[0]==='aaa'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/a[[a]][a]\b/dvy;r.exec('aaaa!')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aaaa!')[0]==='aaa')&&r.lastIndex===4&&r.exec('aaaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/\bc[[a]][b]/v.test('cab')&&/\Bc[[a]][b]/v.test('xcab')&&/c[[a]][b]\b/v.test('cab!')&&/c[[a]][b]\B/v.test('cabx')&&/\bc[[a]][b]\b/v.test('!cab!')&&/\bc[[a]][b]\B/v.test('!cabx')&&/\Bc[[a]][b]\b/v.test('xcab!')&&/\Bc[[a]][b]\B/v.test('xcabx')&&!/\bc[[a]][b]/v.test('xcab')&&!/c[[a]][b]\b/v.test('cabx')&&!/\Bc[[a]][b]/v.test('!cab')&&!/c[[a]][b]\B/v.test('cab!')"#,
    );
    check(
        r#"/\bK[[s]][a]\b/iv.test('KſA!')&&!/\bK[[s]][a]\b/v.test('Ksa!')&&!/\bK[[s]][a]\b/v.test('Kſa!')&&!/\bK[[s]][a]\b/v.test('KsA!')&&/\bK[[s]][a]\b/v.test('Ksa!')&&!/\bK[[s]][a]\b/iv.test('asa!')&&!/\bK[[s]][a]\b/iv.test('Kxa!')&&!/\bK[[s]][a]\b/iv.test('Ksb!')"#,
    );
    check(
        r#"let a=/^\B\$[[\p{Any}]][\p{Any}]\b$/iv,b=/^\B\$[[\p{Any}]][\p{Any}]\b$/v;a.test('$!K')&&a.test('$!ſ')&&!b.test('$!K')&&!b.test('$!ſ')&&a.exec('$!K')[0]==='$!K'"#,
    );
    check(
        r#"/c[a--a][\p{Any}]/v.exec('cab')===null&&/c[[a]][]/v.exec('cab')===null&&/c[a--a][]/v.exec('cab')===null&&/c[[^K]][s]/iv.exec('c!s')[0]==='c!s'&&!/c[[^K]][s]/iv.test('cKs')&&!/c[[^K]][s]/iv.test('cKs')&&/c[[a]][^b]/v.exec('cax')[0]==='cax'&&!/c[[a]][^b]/v.test('cab')"#,
    );
    check(
        r#"/c[[\P{Ll}]][s]/v.test('cAs')&&!/c[[\P{Ll}]][s]/iv.test('cAs')&&!/c[[\P{Ll}]][s]/iv.test('cas')&&/c[[\P{Ll}]][s]/iv.test('c!s')"#,
    );
    check(
        r#"let r=/\b\u{1f600}[[\u{1f601}]][\u{1f600}]\B/dvy;r.lastIndex=2;let m=r.exec('A😀😁😀!');m!==null&&m.index===1&&m[0]==='😀😁😀'&&m.indices[0][0]===1&&m.indices[0][1]===7&&r.lastIndex===7&&r.exec('A😀😁😀!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\b\uD800[[\uD800]][\uD800]\B/dv,m=r.exec('A\uD800\uD800\uD800!');m!==null&&m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&/\uD800[[\uD800]][\uDC00]/v.exec('\uD800\uD800\uDC00')===null"#,
    );
    check(r#"/^\B\[[[\/]][\$]\B$/v.test('[/$')&&/^\bc[[\x61]][\u{62}]\b$/v.test('cab')"#);
    check(
        r#"let r=/^(?<whole>(\bc[[a]][b]\b))$/dvmg,m=r.exec('x\r\ncab\u2028cax');m!==null&&m.index===3&&m[0]==='cab'&&m.indices[2][0]===3&&m.indices[2][1]===6&&r.lastIndex===6&&r.exec('x\r\ncab\u2028cax')===null&&r.lastIndex===0&&!/^\bc[[a]][b]\b$/v.test('cab\n')"#,
    );
    check(
        r#"/\bc[[a]][b]/v.exec('cabx')[0]==='cab'&&/\bc[[a]][b]/v.exec('cabx').index===0&&/\ba[[a]][c]/v.exec('aacx')[0]==='aac'&&/c[[a]][[b]]/v.test('cab')&&/\bc[[a]][[b]]\b/v.test('cab')&&/c[a][b]/v.test('cab')&&/\bc[a][b]\b/v.test('cab')&&/[a]b[c]/v.test('abc')&&/\ba[b]\b/v.test('ab')"#,
    );
    check(
        r#"let r=/(?<whole>(c[[a]][b]\b))/dvg,a=[...'😀cab ccab'.matchAll(r)];a.length===2&&a[0][0]==='cab'&&a[0][1]==='cab'&&a[0][2]==='cab'&&a[0].index===2&&a[0].groups.whole==='cab'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===5&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0"#,
    );
    check(
        r#"'😀cab ccab'.replace(/(?<whole>(c[[a]][b]\b))/vg,'<$1:$<whole>>')==='😀<cab:cab> c<cab:cab>'&&'😀cabx'.split(/(c[[a]][b])/v).join('|')==='😀|cab|x'&&'ccab'.match(/c[[a]][b]/v)[0]==='cab'"#,
    );
    check(
        r#"let seen='',out='😀cab ccab'.replace(/(?<whole>(c[[a]][b]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return '!'});out==='😀! c!'&&seen==='2:cab;7:cab;'"#,
    );
    check(r#"let r=/c[[a]][b]/vg;r.lastIndex=1;'😀ccab'.search(r)===3&&r.lastIndex===1"#);
    check(
        r#"let trace='',r=/\u{1f600}[[\u{1f601}]][a]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5"#,
    );
    check(
        r#"let seen=0,r=/c[[a]][b]/v;r.lastIndex={valueOf(){seen++;return 3}};let m=r.exec('cabcab');m.index===0&&m[0]==='cab'&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/K[[s]][a]/iv,c=new RegExp(r,'dvy');c.source==='K[[s]][a]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſA')===null&&c.lastIndex===0&&r.exec('KſA')[0]==='KſA'"#,
    );
    check(
        r#"let a=/c[[a]][b]/vy,b=/c[[a]][b]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('ccab')}catch(e){x=e instanceof TypeError}try{b.exec('xcab')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let marker=0,caught=false;try{eval('marker=1; /c[a&&][b]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0"#,
    );
    check(r#"/c[a][[b]]/v.test('cab')"#);
    check(
        r#"let r=/c[a][[b]]/v,m=r.exec('cab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[a][[b]]'&&r.lastIndex===0&&!r.test('xab')&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&r.test('cabx')&&r.exec('xcab').index===1"#,
    );
    check(r#"/c[a][c--b]/v.test('cac')"#);
    check(
        r#"let r=/c[a][c--b]/v,m=r.exec('cac');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cac'&&m.index===0&&m.input==='cac'&&m.groups===undefined&&m.indices===undefined&&r.source==='c[a][c--b]'&&r.lastIndex===0&&!r.test('xac')&&!r.test('cxc')&&!r.test('cax')&&!r.test('ca')&&r.test('cacx')&&r.exec('xcac').index===1"#,
    );
    check(r#"/\bc[a][[b]]/v.test('cab')"#);
    check(
        r#"let r=/\bc[a][[b]]/v,m=r.exec('cab');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='cab'&&m.index===0&&m.input==='cab'&&m.groups===undefined&&m.indices===undefined&&r.source==='\\bc[a][[b]]'&&r.lastIndex===0&&!r.test('xab')&&!r.test('cxb')&&!r.test('cax')&&!r.test('ca')&&r.test('cabx')&&!r.test('xcab')&&r.exec('!cab').index===1"#,
    );
    check(r#"/[[a]][b]c/v.test('abc')"#);
    check(
        r#"let r=/[[a]][b]c/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][b]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc').index===1"#,
    );
    check(r#"/[[a]]c[b]/v.test('acb')"#);
    check(
        r#"let r=/[[a]]c[b]/v,m=r.exec('acb');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='acb'&&m.index===0&&m.input==='acb'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]]c[b]'&&r.lastIndex===0&&!r.test('xcb')&&!r.test('axb')&&!r.test('acx')&&!r.test('ac')&&r.exec('acbx')[0]==='acb'&&r.exec('acbx').index===0&&r.exec('xacb').index===1"#,
    );
    for source in [
        r#"/c[[\q{a}]][b]/v.test('cab')"#,
        r#"/c[a--a][\q{b}]/v.test('cab')"#,
        r#"/c[a--a][\p{RGI_Emoji}]/v.test('cab')"#,
        r#"/c[[\p{RGI_Emoji}]][b]/v.test('cab')"#,
        r#"/cc[[a]][b]/v.test('ccab')"#,
        r#"/c[[a]][b]d/v.test('cabd')"#,
        r#"/c[[a]][b][c]/v.test('cabc')"#,
        r#"/c\b[[a]][b]/v.test('cab')"#,
        r#"/c[[a]]\b[b]/v.test('cab')"#,
        r#"/\b^c[[a]][b]/v.test('cab')"#,
        r#"/c[[a]][b]$\b/v.test('cab')"#,
        r#"/(c)[[a]][b]/v.test('cab')"#,
        r#"/c([[a]])[b]/v.test('cab')"#,
        r#"/c[[a]]([b])/v.test('cab')"#,
        r#"/c+[[a]][b]/v.test('cab')"#,
        r#"/c[[a]]+[b]/v.test('cab')"#,
        r#"/c[[a]][b]?/v.test('cab')"#,
        r#"/(?:c[[a]][b])+/v.test('cab')"#,
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
        .eval(r#"var kept=/(?<whole>(a[[a]][a]\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>(a[[a]][a]\\b))'&&kept.lastIndex===1&&kept.exec('aaaa!').groups.whole==='aaa'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    // Explicit opted-in host cancellation only; default JavaScript remains unlimited.
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/a[[a]][c]/vg;var marker=0"#).unwrap();
    let setup = format!("var text='{}b';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn literal_checked_flat_v_set_pair_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete expected JSON/original UTF16 spans are handwritten source data.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>(c[[a]][b]))"#,
            JsString::from_code_units(vec![99, 99, 97, 98]),
            "dvg",
            0,
            r#"{"matches":["cab","cab","cab"],"index":1,"input":"ccab","groups":{"whole":"cab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(c[[a]][b]))"}"#,
        ),
        (
            r#"(?<whole>(a[[a]][b]))"#,
            JsString::from_code_units(vec![97, 97, 97, 98, 33]),
            "dvg",
            0,
            r#"{"matches":["aab","aab","aab"],"index":1,"input":"aaab!","groups":{"whole":"aab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(a[[a]][b]))"}"#,
        ),
        (
            r#"(?<whole>(a[[a]][a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(a[[a]][a]\\b))"}"#,
        ),
        (
            r#"(?<whole>(a[[a]][a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>(a[[a]][a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            1,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(a[[a]][a]\\b))"}"#,
        ),
        (
            r#"(?<whole>(\bc[[a]][b]))"#,
            JsString::from_code_units(vec![99, 97, 98, 120]),
            "dvg",
            0,
            r#"{"matches":["cab","cab","cab"],"index":0,"input":"cabx","groups":{"whole":"cab"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\bc[[a]][b]))"}"#,
        ),
        (
            r#"(?<whole>(\b\u{1f600}[[\u{1f601}]][\u{1f600}]\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56833, 55357, 56832, 33]),
            "dvy",
            2,
            r#"{"matches":["😀😁😀","😀😁😀","😀😁😀"],"index":1,"input":"A😀😁😀!","groups":{"whole":"😀😁😀"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\b\\u{1f600}[[\\u{1f601}]][\\u{1f600}]\\B))"}"#,
        ),
        (
            r#"(?<whole>(\b\uD800[[\uD800]][\uD800]\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b\\uD800[[\\uD800]][\\uD800]\\B))"}"#,
        ),
        (
            r#"^(?<whole>(\B\$[[\p{Any}]][\p{Any}]\b))$"#,
            JsString::from_code_units(vec![36, 33, 8490]),
            "divg",
            0,
            r#"{"matches":["$!K","$!K","$!K"],"index":0,"input":"$!K","groups":{"whole":"$!K"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"^(?<whole>(\\B\\$[[\\p{Any}]][\\p{Any}]\\b))$"}"#,
        ),
        (
            r#"(?<whole>(c[a--a][\p{Any}]))"#,
            JsString::from_code_units(vec![99, 97, 98, 33]),
            "dvg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
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
fn literal_flat_checked_v_set_pair_state_context_and_consumers() {
    check(
        r#"let r=/(?<whole>(c[a][[b]]))/dvg,m=r.exec('ccab');m!==null&&Array.isArray(m)&&m.length===3&&m[0]==='cab'&&m[1]==='cab'&&m[2]==='cab'&&m.index===1&&m.input==='ccab'&&m.groups.whole==='cab'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][0]===1&&m.indices[2][1]===4&&r.source==='(?<whole>(c[a][[b]]))'&&r.lastIndex===4&&r.exec('ccab')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/a[a][[b]]/dvg,m=r.exec('aaab!');m!==null&&m.index===1&&m[0]==='aab'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/a[a][[a]]\b/dvg,m=r.exec('aaaa!');m!==null&&m.index===1&&m[0]==='aaa'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/a[a][[a]]\b/dvy;r.exec('aaaa!')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aaaa!')[0]==='aaa')&&r.lastIndex===4&&r.exec('aaaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/\bc[a][[b]]/v.test('cab')&&/\Bc[a][[b]]/v.test('xcab')&&/c[a][[b]]\b/v.test('cab!')&&/c[a][[b]]\B/v.test('cabx')&&/\bc[a][[b]]\b/v.test('!cab!')&&/\bc[a][[b]]\B/v.test('!cabx')&&/\Bc[a][[b]]\b/v.test('xcab!')&&/\Bc[a][[b]]\B/v.test('xcabx')&&!/\bc[a][[b]]/v.test('xcab')&&!/c[a][[b]]\b/v.test('cabx')&&!/\Bc[a][[b]]/v.test('!cab')&&!/c[a][[b]]\B/v.test('cab!')"#,
    );
    check(
        r#"/\bK[s][[a]]\b/iv.test('KſA!')&&!/\bK[s][[a]]\b/v.test('Ksa!')&&!/\bK[s][[a]]\b/v.test('Kſa!')&&!/\bK[s][[a]]\b/v.test('KsA!')&&/\bK[s][[a]]\b/v.test('Ksa!')&&!/\bK[s][[a]]\b/iv.test('asa!')&&!/\bK[s][[a]]\b/iv.test('Kxa!')&&!/\bK[s][[a]]\b/iv.test('Ksb!')"#,
    );
    check(
        r#"let a=/^\B\$[\p{Any}][[\p{Any}]]\b$/iv,b=/^\B\$[\p{Any}][[\p{Any}]]\b$/v;a.test('$!K')&&a.test('$!ſ')&&!b.test('$!K')&&!b.test('$!ſ')&&a.exec('$!K')[0]==='$!K'"#,
    );
    check(
        r#"/c[][[\p{Any}]]/v.exec('cab')===null&&/c[a][a--a]/v.exec('cab')===null&&/c[][a--a]/v.exec('cab')===null&&/c[^K][[s]]/iv.exec('c!s')[0]==='c!s'&&!/c[^K][[s]]/iv.test('cKs')&&!/c[^K][[s]]/iv.test('cKs')&&/c[a][[^b]]/v.exec('cax')[0]==='cax'&&!/c[a][[^b]]/v.test('cab')"#,
    );
    check(
        r#"/c[\P{Ll}][[s]]/v.test('cAs')&&!/c[\P{Ll}][[s]]/iv.test('cAs')&&!/c[\P{Ll}][[s]]/iv.test('cas')&&/c[\P{Ll}][[s]]/iv.test('c!s')"#,
    );
    check(
        r#"let r=/\b\u{1f600}[\u{1f601}][[\u{1f600}]]\B/dvy;r.lastIndex=2;let m=r.exec('A😀😁😀!');m!==null&&m.index===1&&m[0]==='😀😁😀'&&m.indices[0][0]===1&&m.indices[0][1]===7&&r.lastIndex===7&&r.exec('A😀😁😀!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\b\uD800[\uD800][[\uD800]]\B/dv,m=r.exec('A\uD800\uD800\uD800!');m!==null&&m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&/\uD800[\uD800][[\uDC00]]/v.exec('\uD800\uD800\uDC00')===null"#,
    );
    check(r#"/^\B\[[\/][[\$]]\B$/v.test('[/$')&&/^\bc[\x61][[\u{62}]]\b$/v.test('cab')"#);
    check(
        r#"let r=/^(?<whole>(\bc[a][[b]]\b))$/dvmg,m=r.exec('x\r\ncab\u2028cax');m!==null&&m.index===3&&m[0]==='cab'&&m.indices[2][0]===3&&m.indices[2][1]===6&&r.lastIndex===6&&r.exec('x\r\ncab\u2028cax')===null&&r.lastIndex===0&&!/^\bc[a][[b]]\b$/v.test('cab\n')"#,
    );
    check(
        r#"/\bc[a][[b]]/v.exec('cabx')[0]==='cab'&&/\bc[a][[b]]/v.exec('cabx').index===0&&/\ba[[a]][c]/v.exec('aacx')[0]==='aac'&&/c[[a]][[b]]/v.test('cab')&&/\bc[[a]][[b]]\b/v.test('cab')&&/c[a][b]/v.test('cab')&&/\bc[a][b]\b/v.test('cab')&&/[a]b[c]/v.test('abc')&&/\ba[b]\b/v.test('ab')&&/c[[a]][b]/v.test('cab')&&/\bc[[a]][b]/v.test('cabx')"#,
    );
    check(
        r#"let r=/(?<whole>(c[a][[b]]\b))/dvg,a=[...'😀cab ccab'.matchAll(r)];a.length===2&&a[0][0]==='cab'&&a[0][1]==='cab'&&a[0][2]==='cab'&&a[0].index===2&&a[0].groups.whole==='cab'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===5&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0"#,
    );
    check(
        r#"'😀cab ccab'.replace(/(?<whole>(c[a][[b]]\b))/vg,'<$1:$<whole>>')==='😀<cab:cab> c<cab:cab>'&&'😀cabx'.split(/(c[a][[b]])/v).join('|')==='😀|cab|x'&&'ccab'.match(/c[a][[b]]/v)[0]==='cab'"#,
    );
    check(
        r#"let seen='',out='😀cab ccab'.replace(/(?<whole>(c[a][[b]]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return '!'});out==='😀! c!'&&seen==='2:cab;7:cab;'"#,
    );
    check(r#"let r=/c[a][[b]]/vg;r.lastIndex=1;'😀ccab'.search(r)===3&&r.lastIndex===1"#);
    check(
        r#"let trace='',r=/\u{1f600}[\u{1f601}][[a]]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5"#,
    );
    check(
        r#"let seen=0,r=/c[a][[b]]/v;r.lastIndex={valueOf(){seen++;return 3}};let m=r.exec('cabcab');m.index===0&&m[0]==='cab'&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/K[s][[a]]/iv,c=new RegExp(r,'dvy');c.source==='K[s][[a]]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſA')===null&&c.lastIndex===0&&r.exec('KſA')[0]==='KſA'"#,
    );
    check(
        r#"let a=/c[a][[b]]/vy,b=/c[a][[b]]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('ccab')}catch(e){x=e instanceof TypeError}try{b.exec('xcab')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let marker=0,caught=false;try{eval('marker=1; /c[a][b&&]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0"#,
    );
    check(r#"/[[a]][b]c/v.test('abc')"#);
    check(
        r#"let r=/[[a]][b]c/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]][b]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc').index===1"#,
    );
    check(r#"/[a][[b]]c/v.test('abc')"#);
    check(
        r#"let r=/[a][[b]]c/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a][[b]]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc').index===1"#,
    );
    check(r#"/[[a]]c[b]/v.test('acb')"#);
    check(
        r#"let r=/[[a]]c[b]/v,m=r.exec('acb');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='acb'&&m.index===0&&m.input==='acb'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]]c[b]'&&r.lastIndex===0&&!r.test('xcb')&&!r.test('axb')&&!r.test('acx')&&!r.test('ac')&&r.exec('acbx')[0]==='acb'&&r.exec('acbx').index===0&&r.exec('xacb').index===1"#,
    );
    check(r#"/[a]c[[b]]/v.test('acb')"#);
    check(
        r#"let r=/[a]c[[b]]/v,m=r.exec('acb');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='acb'&&m.index===0&&m.input==='acb'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a]c[[b]]'&&r.lastIndex===0&&!r.test('xcb')&&!r.test('axb')&&!r.test('acx')&&!r.test('ac')&&r.exec('acbx')[0]==='acb'&&r.exec('acbx').index===0&&r.exec('xacb').index===1"#,
    );
    for source in [
        r#"/c[\q{a}][[b]]/v.test('cab')"#,
        r#"/c[][\q{b}&&b]/v.test('cab')"#,
        r#"/c[][[\p{RGI_Emoji}]]/v.test('cab')"#,
        r#"/c[\p{RGI_Emoji}][[b]]/v.test('cab')"#,
        r#"/cc[a][[b]]/v.test('ccab')"#,
        r#"/c[a][[b]]d/v.test('cabd')"#,
        r#"/c[a][[b]][c]/v.test('cabc')"#,
        r#"/c\b[a][[b]]/v.test('cab')"#,
        r#"/c[a]\b[[b]]/v.test('cab')"#,
        r#"/\b^c[a][[b]]/v.test('cab')"#,
        r#"/c[a][[b]]$\b/v.test('cab')"#,
        r#"/(c)[a][[b]]/v.test('cab')"#,
        r#"/c([a])[[b]]/v.test('cab')"#,
        r#"/c[a]([[b]])/v.test('cab')"#,
        r#"/c+[a][[b]]/v.test('cab')"#,
        r#"/c[a]+[[b]]/v.test('cab')"#,
        r#"/c[a][[b]]?/v.test('cab')"#,
        r#"/(?:c[a][[b]])+/v.test('cab')"#,
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
        .eval(r#"var kept=/(?<whole>(a[a][[a]]\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>(a[a][[a]]\\b))'&&kept.lastIndex===1&&kept.exec('aaaa!').groups.whole==='aaa'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    // Explicit host cancellation only; input is built before the matching call.
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/a[a][[c]]/vg;var marker=0"#).unwrap();
    let setup = format!("var text='{}b';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn literal_flat_checked_v_set_pair_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete expected JSON/original UTF16 spans are handwritten source data.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>(c[a][[b]]))"#,
            JsString::from_code_units(vec![99, 99, 97, 98]),
            "dvg",
            0,
            r#"{"matches":["cab","cab","cab"],"index":1,"input":"ccab","groups":{"whole":"cab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(c[a][[b]]))"}"#,
        ),
        (
            r#"(?<whole>(a[a][[b]]))"#,
            JsString::from_code_units(vec![97, 97, 97, 98, 33]),
            "dvg",
            0,
            r#"{"matches":["aab","aab","aab"],"index":1,"input":"aaab!","groups":{"whole":"aab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(a[a][[b]]))"}"#,
        ),
        (
            r#"(?<whole>(a[a][[a]]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(a[a][[a]]\\b))"}"#,
        ),
        (
            r#"(?<whole>(a[a][[a]]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>(a[a][[a]]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            1,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(a[a][[a]]\\b))"}"#,
        ),
        (
            r#"(?<whole>(\bc[a][[b]]))"#,
            JsString::from_code_units(vec![99, 97, 98, 120]),
            "dvg",
            0,
            r#"{"matches":["cab","cab","cab"],"index":0,"input":"cabx","groups":{"whole":"cab"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\bc[a][[b]]))"}"#,
        ),
        (
            r#"(?<whole>(\b\u{1f600}[\u{1f601}][[\u{1f600}]]\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56833, 55357, 56832, 33]),
            "dvy",
            2,
            r#"{"matches":["😀😁😀","😀😁😀","😀😁😀"],"index":1,"input":"A😀😁😀!","groups":{"whole":"😀😁😀"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\b\\u{1f600}[\\u{1f601}][[\\u{1f600}]]\\B))"}"#,
        ),
        (
            r#"(?<whole>(\b\uD800[\uD800][[\uD800]]\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b\\uD800[\\uD800][[\\uD800]]\\B))"}"#,
        ),
        (
            r#"^(?<whole>(\B\$[\p{Any}][[\p{Any}]]\b))$"#,
            JsString::from_code_units(vec![36, 33, 8490]),
            "divg",
            0,
            r#"{"matches":["$!K","$!K","$!K"],"index":0,"input":"$!K","groups":{"whole":"$!K"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"^(?<whole>(\\B\\$[\\p{Any}][[\\p{Any}]]\\b))$"}"#,
        ),
        (
            r#"(?<whole>(c[][[\p{Any}]]))"#,
            JsString::from_code_units(vec![99, 97, 98, 33]),
            "dvg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
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
fn checked_flat_v_set_pair_literal_state_context_and_consumers() {
    check(
        r#"let r=/(?<whole>([[a]][b]c))/dvg,m=r.exec('aabc');m!==null&&Array.isArray(m)&&m.length===3&&m[0]==='abc'&&m[1]==='abc'&&m[2]==='abc'&&m.index===1&&m.input==='aabc'&&m.groups.whole==='abc'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][0]===1&&m.indices[2][1]===4&&r.source==='(?<whole>([[a]][b]c))'&&r.lastIndex===4&&r.exec('aabc')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[[a]][a]b/dvg,m=r.exec('aaab!');m!==null&&m.index===1&&m[0]==='aab'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/[[a]][a]a\b/dvg,m=r.exec('aaaa!');m!==null&&m.index===1&&m[0]==='aaa'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/[[a]][a]a\b/dvy;r.exec('aaaa!')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aaaa!')[0]==='aaa')&&r.lastIndex===4&&r.exec('aaaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/\b[[a]][b]c/v.test('abc')&&/\B[[a]][b]c/v.test('xabc')&&/[[a]][b]c\b/v.test('abc!')&&/[[a]][b]c\B/v.test('abcx')&&/\b[[a]][b]c\b/v.test('!abc!')&&/\b[[a]][b]c\B/v.test('!abcx')&&/\B[[a]][b]c\b/v.test('xabc!')&&/\B[[a]][b]c\B/v.test('xabcx')&&!/\b[[a]][b]c/v.test('xabc')&&!/[[a]][b]c\b/v.test('abcx')&&!/\B[[a]][b]c/v.test('!abc')&&!/[[a]][b]c\B/v.test('abc!')"#,
    );
    check(
        r#"/\b[[K]][s]a\b/iv.test('KſA!')&&!/\b[[K]][s]a\b/v.test('Ksa!')&&!/\b[[K]][s]a\b/v.test('Kſa!')&&!/\b[[K]][s]a\b/v.test('KsA!')&&/\b[[K]][s]a\b/v.test('Ksa!')&&!/\b[[K]][s]a\b/iv.test('asa!')&&!/\b[[K]][s]a\b/iv.test('Kxa!')&&!/\b[[K]][s]a\b/iv.test('Ksb!')"#,
    );
    check(
        r#"let a=/^\b[[\p{Any}]][\p{Any}]\$\B$/iv,b=/^\b[[\p{Any}]][\p{Any}]\$\B$/v;a.test('K!$')&&a.test('ſ!$')&&!b.test('K!$')&&!b.test('ſ!$')&&a.exec('K!$')[0]==='K!$'"#,
    );
    check(
        r#"/[a--a][\p{Any}]c/v.exec('abc')===null&&/[[a]][]c/v.exec('abc')===null&&/[[^a]][b]c/v.exec('xbc!')[0]==='xbc'&&!/[[^a]][b]c/v.test('abc!')&&/[[a]][^b]c/v.exec('axc!')[0]==='axc'&&!/[[a]][^b]c/v.test('abc!')"#,
    );
    check(
        r#"/[[\P{Ll}]][s]x/v.test('Asx')&&!/[[\P{Ll}]][s]x/iv.test('Asx')&&!/[[\P{Ll}]][s]x/iv.test('asx')&&/[[\P{Ll}]][s]x/iv.test('!sx')"#,
    );
    check(
        r#"let r=/\b[[\u{1f600}]][\u{1f601}]\u{1f600}\B/dvy;r.lastIndex=2;let m=r.exec('A😀😁😀!');m!==null&&m.index===1&&m[0]==='😀😁😀'&&m.indices[0][0]===1&&m.indices[0][1]===7&&r.lastIndex===7&&r.exec('A😀😁😀!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\b[[\uD800]][\uD800]\uD800\B/dv,m=r.exec('A\uD800\uD800\uD800!');m!==null&&m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&/[[\uD83D]][\uDE00]a/v.exec('😀a')===null&&/[[a]][\uD83D]\uDE00/v.exec('a😀')===null"#,
    );
    check(
        r#"/^\b[[a]][\]]\$\B$/v.test('a]$')&&!/^\b[[a]][\]]\$\B$/v.test('a]$\n')&&/^[[\u{61}]][\u{62}]\u{63}$/v.test('abc')&&/^[[\u0061]][\u0062]\u0063$/v.test('abc')"#,
    );
    check(
        r#"let r=/^(?<whole>(\b[[a]][b]c\b))$/dvmg,m=r.exec('x\r\nabc\u2028axc');m!==null&&m.index===3&&m[0]==='abc'&&m.indices[2][0]===3&&m.indices[2][1]===6&&r.lastIndex===6&&r.exec('x\r\nabc\u2028axc')===null&&r.lastIndex===0&&!/^\b[[a]][b]c\b$/v.test('abc\n')"#,
    );
    check(
        r#"/\b[[a]][b]c/v.exec('abcx')[0]==='abc'&&/\b[[a]][b]c/v.exec('abcx').index===0&&/[[a]][b]c\b/v.exec('xabc').index===1&&/[[a]][[b]]c/v.test('abc')&&/\b[[a]][[b]]c\b/v.test('abc')&&/[a]b[c]/v.test('abc')&&/\ba[b]\b/v.test('ab')&&/\b[a]b\b/v.test('ab')&&/[a][b][c]/v.test('abc')"#,
    );
    check(
        r#"let r=/(?<whole>([[a]][b]c\b))/dvg,a=[...'😀abc aabc'.matchAll(r)];a.length===2&&a[0][0]==='abc'&&a[0][1]==='abc'&&a[0][2]==='abc'&&a[0].index===2&&a[0].groups.whole==='abc'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===5&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0"#,
    );
    check(
        r#"'😀abc aabc'.replace(/(?<whole>([[a]][b]c\b))/vg,'<$1:$<whole>>')==='😀<abc:abc> a<abc:abc>'&&'😀abcx'.split(/([[a]][b]c)/v).join('|')==='😀|abc|x'&&'aabc'.match(/[[a]][b]c/v)[0]==='abc'"#,
    );
    check(
        r#"let seen='',out='😀abc aabc'.replace(/(?<whole>([[a]][b]c\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return '!'});out==='😀! a!'&&seen==='2:abc;7:abc;'"#,
    );
    check(r#"let r=/[[a]][b]c/vg;r.lastIndex=1;'😀aabc'.search(r)===3&&r.lastIndex===1"#);
    check(
        r#"let trace='',r=/[[\p{Any}]][\u{1f601}]a/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5"#,
    );
    check(
        r#"let seen=0,r=/[[a]][b]c/v;r.lastIndex={valueOf(){seen++;return 3}};let m=r.exec('abcabc');m.index===0&&m[0]==='abc'&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[[K]][s]a/iv,c=new RegExp(r,'dvy');c.source==='[[K]][s]a'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſA')===null&&c.lastIndex===0&&r.exec('KſA')[0]==='KſA'"#,
    );
    check(
        r#"let a=/[[a]][b]c/vy,b=/[[a]][b]c/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aabc')}catch(e){x=e instanceof TypeError}try{b.exec('xabc')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let marker=0,caught=false;try{eval('marker=1; /[[a]][b&&]c/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0"#,
    );
    check(r#"/[a][[b]]c/v.test('abc')"#);
    check(
        r#"let r=/[a][[b]]c/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a][[b]]c'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc').index===1"#,
    );
    check(r#"/[a][c--b]d/v.test('acd')"#);
    check(
        r#"let r=/[a][c--b]d/v,m=r.exec('acd');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='acd'&&m.index===0&&m.input==='acd'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a][c--b]d'&&r.lastIndex===0&&!r.test('xcd')&&!r.test('axd')&&!r.test('acx')&&!r.test('ac')&&r.exec('acdx')[0]==='acd'&&r.exec('acdx').index===0&&r.exec('xacd').index===1"#,
    );
    check(r#"/[[a]]c[b]/v.test('acb')"#);
    check(
        r#"let r=/[[a]]c[b]/v,m=r.exec('acb');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='acb'&&m.index===0&&m.input==='acb'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]]c[b]'&&r.lastIndex===0&&!r.test('xcb')&&!r.test('axb')&&!r.test('acx')&&!r.test('ac')&&r.exec('acbx')[0]==='acb'&&r.exec('acbx').index===0&&r.exec('xacb').index===1"#,
    );
    check(r#"/[a]c[[b]]/v.test('acb')"#);
    check(
        r#"let r=/[a]c[[b]]/v,m=r.exec('acb');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='acb'&&m.index===0&&m.input==='acb'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a]c[[b]]'&&r.lastIndex===0&&!r.test('xcb')&&!r.test('axb')&&!r.test('acx')&&!r.test('ac')&&r.exec('acbx')[0]==='acb'&&r.exec('acbx').index===0&&r.exec('xacb').index===1"#,
    );
    for source in [
        r#"/[[a]][b]cd/v.test('abcd')"#,
        r#"/[[a]][b][c]d/v.test('abcd')"#,
        r#"/a[[a]][b]c/v.test('aabc')"#,
        r#"/[[a]][b]c[d]/v.test('abcd')"#,
        r#"/([[a]])[b]c/v.test('abc')"#,
        r#"/[[a]]([b])c/v.test('abc')"#,
        r#"/[[a]][b](c)/v.test('abc')"#,
        r#"/[[a]]+[b]c/v.test('abc')"#,
        r#"/[[a]][b]?c/v.test('abc')"#,
        r#"/[[a]][b]c+/v.test('abc')"#,
        r#"/(?:[[a]][b]c)+/v.test('abc')"#,
        r#"/[[a]]\b[b]c/v.test('abc')"#,
        r#"/[[a]][b]\bc/v.test('abc')"#,
        r#"/\b^[[a]][b]c/v.test('abc')"#,
        r#"/[[a]][b]c$\b/v.test('abc')"#,
        r#"/\b\b[[a]][b]c/v.test('abc')"#,
        r#"/[a--a][\p{RGI_Emoji}]c/v.test('')"#,
        r#"/[a--a][\q{b}]c/v.test('')"#,
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
        .eval(r#"var kept=/(?<whole>([[a]][a]a\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([[a]][a]a\\b))'&&kept.lastIndex===1&&kept.exec('aaaa!').groups.whole==='aaa'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[[a]][a]c/vg,marker=0"#).unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn checked_flat_v_set_pair_literal_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete expected JSON and original UTF16 spans were hand-derived before execution.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([[a]][b]c))"#,
            JsString::from_code_units(vec![97, 97, 98, 99]),
            "dvg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":1,"input":"aabc","groups":{"whole":"abc"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]][b]c))"}"#,
        ),
        (
            r#"(?<whole>([[a]][a]b))"#,
            JsString::from_code_units(vec![97, 97, 97, 98, 33]),
            "dvg",
            0,
            r#"{"matches":["aab","aab","aab"],"index":1,"input":"aaab!","groups":{"whole":"aab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]][a]b))"}"#,
        ),
        (
            r#"(?<whole>([[a]][a]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]][a]a\\b))"}"#,
        ),
        (
            r#"(?<whole>([[a]][a]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([[a]][a]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            1,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]][a]a\\b))"}"#,
        ),
        (
            r#"(?<whole>(\b[[a]][b]c))"#,
            JsString::from_code_units(vec![97, 98, 99, 120]),
            "dvg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":0,"input":"abcx","groups":{"whole":"abc"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\b[[a]][b]c))"}"#,
        ),
        (
            r#"(?<whole>(\b[[\u{1f600}]][\u{1f601}]\u{1f600}\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56833, 55357, 56832, 33]),
            "dvy",
            2,
            r#"{"matches":["😀😁😀","😀😁😀","😀😁😀"],"index":1,"input":"A😀😁😀!","groups":{"whole":"😀😁😀"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\b[[\\u{1f600}]][\\u{1f601}]\\u{1f600}\\B))"}"#,
        ),
        (
            r#"(?<whole>(\b[[\uD800]][\uD800]\uD800\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b[[\\uD800]][\\uD800]\\uD800\\B))"}"#,
        ),
        (
            r#"^(?<whole>(\b[[\p{Any}]][\p{Any}]\$\B))$"#,
            JsString::from_code_units(vec![8490, 33, 36]),
            "divg",
            0,
            r#"{"matches":["K!$","K!$","K!$"],"index":0,"input":"K!$","groups":{"whole":"K!$"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"^(?<whole>(\\b[[\\p{Any}]][\\p{Any}]\\$\\B))$"}"#,
        ),
        (
            r#"(?<whole>([a--a][\p{Any}]c))"#,
            JsString::from_code_units(vec![97, 98, 99, 33]),
            "dvg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
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
fn flat_checked_v_set_pair_literal_state_context_and_consumers() {
    check(
        r#"let r=/(?<whole>([a][[b]]c))/dvg,m=r.exec('aabc');m!==null&&Array.isArray(m)&&m.length===3&&m[0]==='abc'&&m[1]==='abc'&&m[2]==='abc'&&m.index===1&&m.input==='aabc'&&m.groups.whole==='abc'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][0]===1&&m.indices[2][1]===4&&r.source==='(?<whole>([a][[b]]c))'&&r.lastIndex===4&&r.exec('aabc')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[a][[a]]b/dvg,m=r.exec('aaab!');m!==null&&m.index===1&&m[0]==='aab'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/[a][[a]]a\b/dvg,m=r.exec('aaaa!');m!==null&&m.index===1&&m[0]==='aaa'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/[a][[a]]a\b/dvy;r.exec('aaaa!')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aaaa!')[0]==='aaa')&&r.lastIndex===4&&r.exec('aaaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/\b[a][[b]]c/v.test('abc')&&/\B[a][[b]]c/v.test('xabc')&&/[a][[b]]c\b/v.test('abc!')&&/[a][[b]]c\B/v.test('abcx')&&/\b[a][[b]]c\b/v.test('!abc!')&&/\b[a][[b]]c\B/v.test('!abcx')&&/\B[a][[b]]c\b/v.test('xabc!')&&/\B[a][[b]]c\B/v.test('xabcx')&&!/\b[a][[b]]c/v.test('xabc')&&!/[a][[b]]c\b/v.test('abcx')&&!/\B[a][[b]]c/v.test('!abc')&&!/[a][[b]]c\B/v.test('abc!')"#,
    );
    check(
        r#"/\b[K][[s]]a\b/iv.test('KſA!')&&!/\b[K][[s]]a\b/v.test('Ksa!')&&!/\b[K][[s]]a\b/v.test('Kſa!')&&!/\b[K][[s]]a\b/v.test('KsA!')&&/\b[K][[s]]a\b/v.test('Ksa!')&&!/\b[K][[s]]a\b/iv.test('asa!')&&!/\b[K][[s]]a\b/iv.test('Kxa!')&&!/\b[K][[s]]a\b/iv.test('Ksb!')"#,
    );
    check(
        r#"let a=/^\b[\p{Any}][[\p{Any}]]\$\B$/iv,b=/^\b[\p{Any}][[\p{Any}]]\$\B$/v;a.test('K!$')&&a.test('ſ!$')&&!b.test('K!$')&&!b.test('ſ!$')&&a.exec('K!$')[0]==='K!$'"#,
    );
    check(
        r#"/[][[\p{Any}]]c/v.exec('abc')===null&&/[a][a--a]c/v.exec('abc')===null&&/[^a][[b]]c/v.exec('xbc!')[0]==='xbc'&&!/[^a][[b]]c/v.test('abc!')&&/[a][[^b]]c/v.exec('axc!')[0]==='axc'&&!/[a][[^b]]c/v.test('abc!')"#,
    );
    check(
        r#"/[\P{Ll}][[s]]x/v.test('Asx')&&!/[\P{Ll}][[s]]x/iv.test('Asx')&&!/[\P{Ll}][[s]]x/iv.test('asx')&&/[\P{Ll}][[s]]x/iv.test('!sx')"#,
    );
    check(
        r#"let r=/\b[\u{1f600}][[\u{1f601}]]\u{1f600}\B/dvy;r.lastIndex=2;let m=r.exec('A😀😁😀!');m!==null&&m.index===1&&m[0]==='😀😁😀'&&m.indices[0][0]===1&&m.indices[0][1]===7&&r.lastIndex===7&&r.exec('A😀😁😀!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\b[\uD800][[\uD800]]\uD800\B/dv,m=r.exec('A\uD800\uD800\uD800!');m!==null&&m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&/[\uD83D][[\uDE00]]a/v.exec('😀a')===null&&/[a][[\uD83D]]\uDE00/v.exec('a😀')===null"#,
    );
    check(
        r#"/^\b[a][[\]]]\$\B$/v.test('a]$')&&!/^\b[a][[\]]]\$\B$/v.test('a]$\n')&&/^[\u{61}][[\u{62}]]\u{63}$/v.test('abc')&&/^[\u0061][[\u0062]]\u0063$/v.test('abc')"#,
    );
    check(
        r#"let r=/^(?<whole>(\b[a][[b]]c\b))$/dvmg,m=r.exec('x\r\nabc\u2028axc');m!==null&&m.index===3&&m[0]==='abc'&&m.indices[2][0]===3&&m.indices[2][1]===6&&r.lastIndex===6&&r.exec('x\r\nabc\u2028axc')===null&&r.lastIndex===0&&!/^\b[a][[b]]c\b$/v.test('abc\n')"#,
    );
    check(
        r#"/\b[a][[b]]c/v.exec('abcx')[0]==='abc'&&/\b[a][[b]]c/v.exec('abcx').index===0&&/[a][[b]]c\b/v.exec('xabc').index===1&&/[[a]][[b]]c/v.test('abc')&&/\b[[a]][[b]]c\b/v.test('abc')&&/[a]b[c]/v.test('abc')&&/\ba[b]\b/v.test('ab')&&/\b[a]b\b/v.test('ab')&&/[a][b][c]/v.test('abc')"#,
    );
    check(
        r#"let r=/(?<whole>([a][[b]]c\b))/dvg,a=[...'😀abc aabc'.matchAll(r)];a.length===2&&a[0][0]==='abc'&&a[0][1]==='abc'&&a[0][2]==='abc'&&a[0].index===2&&a[0].groups.whole==='abc'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===5&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0"#,
    );
    check(
        r#"'😀abc aabc'.replace(/(?<whole>([a][[b]]c\b))/vg,'<$1:$<whole>>')==='😀<abc:abc> a<abc:abc>'&&'😀abcx'.split(/([a][[b]]c)/v).join('|')==='😀|abc|x'&&'aabc'.match(/[a][[b]]c/v)[0]==='abc'"#,
    );
    check(
        r#"let seen='',out='😀abc aabc'.replace(/(?<whole>([a][[b]]c\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return '!'});out==='😀! a!'&&seen==='2:abc;7:abc;'"#,
    );
    check(r#"let r=/[a][[b]]c/vg;r.lastIndex=1;'😀aabc'.search(r)===3&&r.lastIndex===1"#);
    check(
        r#"let trace='',r=/[\p{Any}][[\u{1f601}]]a/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀😁a!'}});trace==='si'&&m.index===0&&m[0]==='😀😁a'&&r.lastIndex===5"#,
    );
    check(
        r#"let seen=0,r=/[a][[b]]c/v;r.lastIndex={valueOf(){seen++;return 3}};let m=r.exec('abcabc');m.index===0&&m[0]==='abc'&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[K][[s]]a/iv,c=new RegExp(r,'dvy');c.source==='[K][[s]]a'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſA')===null&&c.lastIndex===0&&r.exec('KſA')[0]==='KſA'"#,
    );
    check(
        r#"let a=/[a][[b]]c/vy,b=/[a][[b]]c/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aabc')}catch(e){x=e instanceof TypeError}try{b.exec('xabc')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let marker=0,caught=false;try{eval('marker=1; /[a][b&&]c/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0"#,
    );
    check(r#"/[[a]]c[b]/v.test('acb')"#);
    check(
        r#"let r=/[[a]]c[b]/v,m=r.exec('acb');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='acb'&&m.index===0&&m.input==='acb'&&m.groups===undefined&&m.indices===undefined&&r.source==='[[a]]c[b]'&&r.lastIndex===0&&!r.test('xcb')&&!r.test('axb')&&!r.test('acx')&&!r.test('ac')&&r.exec('acbx')[0]==='acb'&&r.exec('acbx').index===0&&r.exec('xacb').index===1"#,
    );
    check(r#"/[a]c[[b]]/v.test('acb')"#);
    check(
        r#"let r=/[a]c[[b]]/v,m=r.exec('acb');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='acb'&&m.index===0&&m.input==='acb'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a]c[[b]]'&&r.lastIndex===0&&!r.test('xcb')&&!r.test('axb')&&!r.test('acx')&&!r.test('ac')&&r.exec('acbx')[0]==='acb'&&r.exec('acbx').index===0&&r.exec('xacb').index===1"#,
    );
    for source in [
        r#"/[a][[b]]cd/v.test('abcd')"#,
        r#"/[a][[b]][c]d/v.test('abcd')"#,
        r#"/a[a][[b]]c/v.test('aabc')"#,
        r#"/[a][[b]]c[d]/v.test('abcd')"#,
        r#"/([a])[[b]]c/v.test('abc')"#,
        r#"/[a]([[b]])c/v.test('abc')"#,
        r#"/[a][[b]](c)/v.test('abc')"#,
        r#"/[a]+[[b]]c/v.test('abc')"#,
        r#"/[a][[b]]?c/v.test('abc')"#,
        r#"/[a][[b]]c+/v.test('abc')"#,
        r#"/(?:[a][[b]]c)+/v.test('abc')"#,
        r#"/[a]\b[[b]]c/v.test('abc')"#,
        r#"/[a][[b]]\bc/v.test('abc')"#,
        r#"/\b^[a][[b]]c/v.test('abc')"#,
        r#"/[a][[b]]c$\b/v.test('abc')"#,
        r#"/\b\b[a][[b]]c/v.test('abc')"#,
        r#"/[][\p{RGI_Emoji}&&a]c/v.test('')"#,
        r#"/[][\q{b}&&b]c/v.test('')"#,
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
        .eval(r#"var kept=/(?<whole>([a][[a]]a\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([a][[a]]a\\b))'&&kept.lastIndex===1&&kept.exec('aaaa!').groups.whole==='aaa'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[a][[a]]c/vg,marker=0"#).unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn flat_checked_v_set_pair_literal_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete expected JSON and original UTF16 spans were hand-derived before execution.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([a][[b]]c))"#,
            JsString::from_code_units(vec![97, 97, 98, 99]),
            "dvg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":1,"input":"aabc","groups":{"whole":"abc"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a][[b]]c))"}"#,
        ),
        (
            r#"(?<whole>([a][[a]]b))"#,
            JsString::from_code_units(vec![97, 97, 97, 98, 33]),
            "dvg",
            0,
            r#"{"matches":["aab","aab","aab"],"index":1,"input":"aaab!","groups":{"whole":"aab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a][[a]]b))"}"#,
        ),
        (
            r#"(?<whole>([a][[a]]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a][[a]]a\\b))"}"#,
        ),
        (
            r#"(?<whole>([a][[a]]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([a][[a]]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            1,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a][[a]]a\\b))"}"#,
        ),
        (
            r#"(?<whole>(\b[a][[b]]c))"#,
            JsString::from_code_units(vec![97, 98, 99, 120]),
            "dvg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":0,"input":"abcx","groups":{"whole":"abc"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\b[a][[b]]c))"}"#,
        ),
        (
            r#"(?<whole>(\b[\u{1f600}][[\u{1f601}]]\u{1f600}\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56833, 55357, 56832, 33]),
            "dvy",
            2,
            r#"{"matches":["😀😁😀","😀😁😀","😀😁😀"],"index":1,"input":"A😀😁😀!","groups":{"whole":"😀😁😀"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\b[\\u{1f600}][[\\u{1f601}]]\\u{1f600}\\B))"}"#,
        ),
        (
            r#"(?<whole>(\b[\uD800][[\uD800]]\uD800\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b[\\uD800][[\\uD800]]\\uD800\\B))"}"#,
        ),
        (
            r#"^(?<whole>(\b[\p{Any}][[\p{Any}]]\$\B))$"#,
            JsString::from_code_units(vec![8490, 33, 36]),
            "divg",
            0,
            r#"{"matches":["K!$","K!$","K!$"],"index":0,"input":"K!$","groups":{"whole":"K!$"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"^(?<whole>(\\b[\\p{Any}][[\\p{Any}]]\\$\\B))$"}"#,
        ),
        (
            r#"(?<whole>([][[\p{Any}]]c))"#,
            JsString::from_code_units(vec![97, 98, 99, 33]),
            "dvg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
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
fn checked_v_set_literal_flat_v_set_state_context_and_consumers() {
    check(
        r#"let r=/(?<whole>([[a]]b[c]))/dvg,m=r.exec('aabc');m!==null&&Array.isArray(m)&&m.length===3&&m[0]==='abc'&&m[1]==='abc'&&m[2]==='abc'&&m.index===1&&m.input==='aabc'&&m.groups.whole==='abc'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][0]===1&&m.indices[2][1]===4&&r.source==='(?<whole>([[a]]b[c]))'&&r.lastIndex===4&&r.exec('aabc')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[[a]]a[b]/dvg,m=r.exec('aaab!');m!==null&&m.index===1&&m[0]==='aab'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/[[a]]a[a]\b/dvg,m=r.exec('aaaa!');m!==null&&m.index===1&&m[0]==='aaa'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/[[a]]a[a]\b/dvy;r.exec('aaaa!')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aaaa!')[0]==='aaa')&&r.lastIndex===4&&r.exec('aaaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/\b[[a]]b[c]/v.test('abc')&&/\B[[a]]b[c]/v.test('xabc')&&/[[a]]b[c]\b/v.test('abc!')&&/[[a]]b[c]\B/v.test('abcx')&&/\b[[a]]b[c]\b/v.test('!abc!')&&/\b[[a]]b[c]\B/v.test('!abcx')&&/\B[[a]]b[c]\b/v.test('xabc!')&&/\B[[a]]b[c]\B/v.test('xabcx')&&!/\b[[a]]b[c]/v.test('xabc')&&!/[[a]]b[c]\b/v.test('abcx')&&!/\B[[a]]b[c]/v.test('!abc')&&!/[[a]]b[c]\B/v.test('abc!')"#,
    );
    check(
        r#"/\b[[K]]s[a]\b/iv.test('KſA!')&&!/\b[[K]]s[a]\b/v.test('Ksa!')&&!/\b[[K]]s[a]\b/v.test('Kſa!')&&!/\b[[K]]s[a]\b/v.test('KsA!')&&/\b[[K]]s[a]\b/v.test('Ksa!')&&!/\b[[K]]s[a]\b/iv.test('asa!')&&!/\b[[K]]s[a]\b/iv.test('Kxa!')&&!/\b[[K]]s[a]\b/iv.test('Ksb!')"#,
    );
    check(
        r#"let a=/^\B[[\p{Any}]]\$[\p{Any}]\b$/iv,b=/^\B[[\p{Any}]]\$[\p{Any}]\b$/v;a.test('!$K')&&a.test('!$ſ')&&!b.test('!$K')&&!b.test('!$ſ')&&a.exec('!$K')[0]==='!$K'"#,
    );
    check(
        r#"/[a--a]b[c]/v.exec('abc')===null&&/[[a]]b[]/v.exec('abc')===null&&/[[^a]]b[c]/v.exec('xbc!')[0]==='xbc'&&!/[[^a]]b[c]/v.test('abc!')&&/[[a]]b[^c]/v.exec('abx!')[0]==='abx'&&!/[[a]]b[^c]/v.test('abc!')"#,
    );
    check(
        r#"/[[\P{Ll}]]b[c]/v.test('Abc')&&!/[[\P{Ll}]]b[c]/iv.test('Abc')&&!/[[\P{Ll}]]b[c]/iv.test('abc')&&/[[\P{Ll}]]b[c]/iv.test('!bc')"#,
    );
    check(
        r#"let r=/\b[[\u{1f600}]]\u{1f601}[\u{1f600}]\B/dvy;r.lastIndex=2;let m=r.exec('A😀😁😀!');m!==null&&m.index===1&&m[0]==='😀😁😀'&&m.indices[0][0]===1&&m.indices[0][1]===7&&r.lastIndex===7&&r.exec('A😀😁😀!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\b[[\uD800]]\uD800[\uD800]\B/dv,m=r.exec('A\uD800\uD800\uD800!');m!==null&&m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&/[[\uD83D]]\uDE00[a]/v.exec('😀a')===null&&/[[a]]\uD83D[\uDE00]/v.exec('a😀')===null"#,
    );
    check(
        r#"/^\B[[\]]]\$[!]\B$/v.test(']$!')&&!/^\B[[\]]]\$[!]\B$/v.test(']$!\n')&&/^[[\u{61}]]\u{62}[\u{63}]$/v.test('abc')&&/^[[\u0061]]\u0062[\u0063]$/v.test('abc')"#,
    );
    check(
        r#"let r=/^(?<whole>(\b[[a]]b[c]\b))$/dvmg,m=r.exec('x\r\nabc\u2028abx');m!==null&&m.index===3&&m[0]==='abc'&&m.indices[2][0]===3&&m.indices[2][1]===6&&r.lastIndex===6&&r.exec('x\r\nabc\u2028abx')===null&&r.lastIndex===0&&!/^\b[[a]]b[c]\b$/v.test('abc\n')"#,
    );
    check(
        r#"/\b[[a]]b[c]/v.exec('abcx')[0]==='abc'&&/\b[[a]]b[c]/v.exec('abcx').index===0&&/[[a]]b[c]\b/v.exec('xabc').index===1&&/[[a]]b[[c]]/v.test('abc')&&/\b[[a]]b[[c]]\b/v.test('abc')&&/[a]b[c]/v.test('abc')&&/[[a]][b]c/v.test('abc')&&/[a][[b]]c/v.test('abc')&&/a[[b]][c]/v.test('abc')&&/a[b][[c]]/v.test('abc')"#,
    );
    check(
        r#"let r=/(?<whole>([[a]]b[c]\b))/dvg,a=[...'😀abc aabc'.matchAll(r)];a.length===2&&a[0][0]==='abc'&&a[0][1]==='abc'&&a[0][2]==='abc'&&a[0].index===2&&a[0].groups.whole==='abc'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===5&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0"#,
    );
    check(
        r#"'😀abc aabc'.replace(/(?<whole>([[a]]b[c]\b))/vg,'<$1:$<whole>>')==='😀<abc:abc> a<abc:abc>'&&'😀abcx'.split(/([[a]]b[c])/v).join('|')==='😀|abc|x'&&'aabc'.match(/[[a]]b[c]/v)[0]==='abc'"#,
    );
    check(
        r#"let seen='',out='😀abc aabc'.replace(/(?<whole>([[a]]b[c]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return '!'});out==='😀! a!'&&seen==='2:abc;7:abc;'"#,
    );
    check(r#"let r=/[[a]]b[c]/vg;r.lastIndex=1;'😀aabc'.search(r)===3&&r.lastIndex===1"#);
    check(
        r#"let trace='',r=/[[\p{Any}]]\$[\p{Any}]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀$K'}});trace==='si'&&m.index===0&&m[0]==='😀$K'&&r.lastIndex===4"#,
    );
    check(
        r#"let seen=0,r=/[[a]]b[c]/v;r.lastIndex={valueOf(){seen++;return 3}};let m=r.exec('aabc');m.index===1&&m[0]==='abc'&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[[K]]s[a]/iv,c=new RegExp(r,'dvy');c.source==='[[K]]s[a]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſA')===null&&c.lastIndex===0&&r.exec('KſA')[0]==='KſA'"#,
    );
    check(
        r#"let a=/[[a]]b[c]/vy,b=/[[a]]b[c]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aabc')}catch(e){x=e instanceof TypeError}try{b.exec('xabc')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let marker=0,caught=false;try{eval('marker=1; /[a&&]b[c]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0"#,
    );
    check(r#"/[a]b[[c]]/v.test('abc')"#);
    check(
        r#"let r=/[a]b[[c]]/v,m=r.exec('abc');m!==null&&Array.isArray(m)&&m.length===1&&m[0]==='abc'&&m.index===0&&m.input==='abc'&&m.groups===undefined&&m.indices===undefined&&r.source==='[a]b[[c]]'&&r.lastIndex===0&&!r.test('xbc')&&!r.test('axc')&&!r.test('abx')&&!r.test('ab')&&r.exec('abcx')[0]==='abc'&&r.exec('abcx').index===0&&r.exec('xabc').index===1"#,
    );
    for source in [
        r#"/[[a]]b[c]d/v.test('abcd')"#,
        r#"/x[[a]]b[c]/v.test('xabc')"#,
        r#"/[[a]]b[c][d]/v.test('abcd')"#,
        r#"/([[a]])b[c]/v.test('abc')"#,
        r#"/[[a]](b)[c]/v.test('abc')"#,
        r#"/[[a]]b([c])/v.test('abc')"#,
        r#"/(?:[[a]]b[c])+/v.test('abc')"#,
        r#"/[[a]]+b[c]/v.test('abc')"#,
        r#"/[[a]]b[c]?/v.test('abc')"#,
        r#"/[[a]]\bb[c]/v.test('abc')"#,
        r#"/[[a]]b\b[c]/v.test('abc')"#,
        r#"/\b^[[a]]b[c]/v.test('abc')"#,
        r#"/[[a]]b[c]$\b/v.test('abc')"#,
        r#"/\b\b[[a]]b[c]/v.test('abc')"#,
        r#"/[[a]]b[c]\b\b/v.test('abc')"#,
        r#"/[\q{a}&&a]b[]/v.test('')"#,
        r#"/[a--a]b[\p{RGI_Emoji}]/v.test('')"#,
        r#"/[a--a]b[\q{c}]/v.test('')"#,
        r#"/[\p{RGI_Emoji}&&a]b[]/v.test('')"#,
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
        .eval(r#"var kept=/(?<whole>([[a]]a[a]\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([[a]]a[a]\\b))'&&kept.lastIndex===1&&kept.exec('aaaa!').groups.whole==='aaa'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[[a]]a[c]/vg,marker=0"#).unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn checked_v_set_literal_flat_v_set_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full JSON and original UTF16 spans were hand-derived before Runtime source.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([[a]]b[c]))"#,
            JsString::from_code_units(vec![97, 97, 98, 99]),
            "dvg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":1,"input":"aabc","groups":{"whole":"abc"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]]b[c]))"}"#,
        ),
        (
            r#"(?<whole>([[a]]a[b]))"#,
            JsString::from_code_units(vec![97, 97, 97, 98, 33]),
            "dvg",
            0,
            r#"{"matches":["aab","aab","aab"],"index":1,"input":"aaab!","groups":{"whole":"aab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]]a[b]))"}"#,
        ),
        (
            r#"(?<whole>([[a]]a[a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]]a[a]\\b))"}"#,
        ),
        (
            r#"(?<whole>([[a]]a[a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([[a]]a[a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            1,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([[a]]a[a]\\b))"}"#,
        ),
        (
            r#"(?<whole>(\b[[a]]b[c]))"#,
            JsString::from_code_units(vec![97, 98, 99, 120]),
            "dvg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":0,"input":"abcx","groups":{"whole":"abc"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\b[[a]]b[c]))"}"#,
        ),
        (
            r#"(?<whole>(\b[[\u{1f600}]]\u{1f601}[\u{1f600}]\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56833, 55357, 56832, 33]),
            "dvy",
            2,
            r#"{"matches":["😀😁😀","😀😁😀","😀😁😀"],"index":1,"input":"A😀😁😀!","groups":{"whole":"😀😁😀"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\b[[\\u{1f600}]]\\u{1f601}[\\u{1f600}]\\B))"}"#,
        ),
        (
            r#"(?<whole>(\b[[\uD800]]\uD800[\uD800]\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b[[\\uD800]]\\uD800[\\uD800]\\B))"}"#,
        ),
        (
            r#"^(?<whole>(\B[[\p{Any}]]\$[\p{Any}]\b))$"#,
            JsString::from_code_units(vec![33, 36, 8490]),
            "divg",
            0,
            r#"{"matches":["!$K","!$K","!$K"],"index":0,"input":"!$K","groups":{"whole":"!$K"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"^(?<whole>(\\B[[\\p{Any}]]\\$[\\p{Any}]\\b))$"}"#,
        ),
        (
            r#"(?<whole>([a--a]c[\p{Any}]))"#,
            JsString::from_code_units(vec![97, 99, 98, 33]),
            "dvg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
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
fn flat_v_set_literal_checked_v_set_state_context_and_consumers() {
    check(
        r#"let r=/(?<whole>([a]b[[c]]))/dvg,m=r.exec('aabc');m!==null&&Array.isArray(m)&&m.length===3&&m[0]==='abc'&&m[1]==='abc'&&m[2]==='abc'&&m.index===1&&m.input==='aabc'&&m.groups.whole==='abc'&&m.indices.groups.whole===m.indices[1]&&m.indices[2][0]===1&&m.indices[2][1]===4&&r.source==='(?<whole>([a]b[[c]]))'&&r.lastIndex===4&&r.exec('aabc')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[a]a[[b]]/dvg,m=r.exec('aaab!');m!==null&&m.index===1&&m[0]==='aab'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/[a]a[[a]]\b/dvg,m=r.exec('aaaa!');m!==null&&m.index===1&&m[0]==='aaa'&&m.indices[0][1]===4&&r.lastIndex===4"#,
    );
    check(
        r#"let r=/[a]a[[a]]\b/dvy;r.exec('aaaa!')===null&&r.lastIndex===0&&(r.lastIndex=1,r.exec('aaaa!')[0]==='aaa')&&r.lastIndex===4&&r.exec('aaaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/\b[a]b[[c]]/v.test('abc')&&/\B[a]b[[c]]/v.test('xabc')&&/[a]b[[c]]\b/v.test('abc!')&&/[a]b[[c]]\B/v.test('abcx')&&/\b[a]b[[c]]\b/v.test('!abc!')&&/\b[a]b[[c]]\B/v.test('!abcx')&&/\B[a]b[[c]]\b/v.test('xabc!')&&/\B[a]b[[c]]\B/v.test('xabcx')&&!/\b[a]b[[c]]/v.test('xabc')&&!/[a]b[[c]]\b/v.test('abcx')&&!/\B[a]b[[c]]/v.test('!abc')&&!/[a]b[[c]]\B/v.test('abc!')"#,
    );
    check(
        r#"/\b[K]s[[a]]\b/iv.test('KſA!')&&!/\b[K]s[[a]]\b/v.test('Ksa!')&&!/\b[K]s[[a]]\b/v.test('Kſa!')&&!/\b[K]s[[a]]\b/v.test('KsA!')&&/\b[K]s[[a]]\b/v.test('Ksa!')&&!/\b[K]s[[a]]\b/iv.test('asa!')&&!/\b[K]s[[a]]\b/iv.test('Kxa!')&&!/\b[K]s[[a]]\b/iv.test('Ksb!')"#,
    );
    check(
        r#"let a=/^\B[\p{Any}]\$[[\p{Any}]]\b$/iv,b=/^\B[\p{Any}]\$[[\p{Any}]]\b$/v;a.test('!$K')&&a.test('!$ſ')&&!b.test('!$K')&&!b.test('!$ſ')&&a.exec('!$K')[0]==='!$K'"#,
    );
    check(
        r#"/[]b[[c]]/v.exec('abc')===null&&/[a]b[a--a]/v.exec('abc')===null&&/[^a]b[[c]]/v.exec('xbc!')[0]==='xbc'&&!/[^a]b[[c]]/v.test('abc!')&&/[a]b[[^c]]/v.exec('abx!')[0]==='abx'&&!/[a]b[[^c]]/v.test('abc!')"#,
    );
    check(
        r#"/[\P{Ll}]b[[c]]/v.test('Abc')&&!/[\P{Ll}]b[[c]]/iv.test('Abc')&&!/[\P{Ll}]b[[c]]/iv.test('abc')&&/[\P{Ll}]b[[c]]/iv.test('!bc')"#,
    );
    check(
        r#"let r=/\b[\u{1f600}]\u{1f601}[[\u{1f600}]]\B/dvy;r.lastIndex=2;let m=r.exec('A😀😁😀!');m!==null&&m.index===1&&m[0]==='😀😁😀'&&m.indices[0][0]===1&&m.indices[0][1]===7&&r.lastIndex===7&&r.exec('A😀😁😀!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\b[\uD800]\uD800[[\uD800]]\B/dv,m=r.exec('A\uD800\uD800\uD800!');m!==null&&m.index===1&&m[0]==='\uD800\uD800\uD800'&&m.indices[0][1]===4&&/[\uD83D]\uDE00[[a]]/v.exec('😀a')===null&&/[a]\uD83D[[\uDE00]]/v.exec('a😀')===null"#,
    );
    check(
        r#"/^\B[\]]\$[[!]]\B$/v.test(']$!')&&!/^\B[\]]\$[[!]]\B$/v.test(']$!\n')&&/^[\u{61}]\u{62}[[\u{63}]]$/v.test('abc')&&/^[\u0061]\u0062[[\u0063]]$/v.test('abc')"#,
    );
    check(
        r#"let r=/^(?<whole>(\b[a]b[[c]]\b))$/dvmg,m=r.exec('x\r\nabc\u2028abx');m!==null&&m.index===3&&m[0]==='abc'&&m.indices[2][0]===3&&m.indices[2][1]===6&&r.lastIndex===6&&r.exec('x\r\nabc\u2028abx')===null&&r.lastIndex===0&&!/^\b[a]b[[c]]\b$/v.test('abc\n')"#,
    );
    check(
        r#"/\b[a]b[[c]]/v.exec('abcx')[0]==='abc'&&/\b[a]b[[c]]/v.exec('abcx').index===0&&/[a]b[[c]]\b/v.exec('xabc').index===1&&/[[a]]b[[c]]/v.test('abc')&&/\b[[a]]b[[c]]\b/v.test('abc')&&/[a]b[c]/v.test('abc')&&/[[a]][b]c/v.test('abc')&&/[a][[b]]c/v.test('abc')&&/a[[b]][c]/v.test('abc')&&/a[b][[c]]/v.test('abc')&&/[[a]]b[c]/v.test('abc')"#,
    );
    check(
        r#"let r=/(?<whole>([a]b[[c]]\b))/dvg,a=[...'😀abc aabc'.matchAll(r)];a.length===2&&a[0][0]==='abc'&&a[0][1]==='abc'&&a[0][2]==='abc'&&a[0].index===2&&a[0].groups.whole==='abc'&&a[0].indices.groups.whole===a[0].indices[1]&&a[0].indices[2][1]===5&&a[1].index===7&&a[1].indices[2][1]===10&&r.lastIndex===0"#,
    );
    check(
        r#"'😀abc aabc'.replace(/(?<whole>([a]b[[c]]\b))/vg,'<$1:$<whole>>')==='😀<abc:abc> a<abc:abc>'&&'😀abcx'.split(/([a]b[[c]])/v).join('|')==='😀|abc|x'&&'aabc'.match(/[a]b[[c]]/v)[0]==='abc'"#,
    );
    check(
        r#"let seen='',out='😀abc aabc'.replace(/(?<whole>([a]b[[c]]\b))/vg,(m,c1,c2,p,text,g)=>{seen+=p+':'+g.whole+';';return '!'});out==='😀! a!'&&seen==='2:abc;7:abc;'"#,
    );
    check(r#"let r=/[a]b[[c]]/vg;r.lastIndex=1;'😀aabc'.search(r)===3&&r.lastIndex===1"#);
    check(
        r#"let trace='',r=/[\p{Any}]\$[[\p{Any}]]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return '😀$K'}});trace==='si'&&m.index===0&&m[0]==='😀$K'&&r.lastIndex===4"#,
    );
    check(
        r#"let seen=0,r=/[a]b[[c]]/v;r.lastIndex={valueOf(){seen++;return 3}};let m=r.exec('aabc');m.index===1&&m[0]==='abc'&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[K]s[[a]]/iv,c=new RegExp(r,'dvy');c.source==='[K]s[[a]]'&&c.unicodeSets&&!c.ignoreCase&&c.exec('KſA')===null&&c.lastIndex===0&&r.exec('KſA')[0]==='KſA'"#,
    );
    check(
        r#"let a=/[a]b[[c]]/vy,b=/[a]b[[c]]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aabc')}catch(e){x=e instanceof TypeError}try{b.exec('xabc')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let marker=0,caught=false;try{eval('marker=1; /[a]b[c&&]/v;')}catch(e){caught=e instanceof SyntaxError}caught&&marker===0"#,
    );
    for source in [
        r#"/[a]b[[c]]d/v.test('abcd')"#,
        r#"/x[a]b[[c]]/v.test('xabc')"#,
        r#"/[a]b[[c]][d]/v.test('abcd')"#,
        r#"/([a])b[[c]]/v.test('abc')"#,
        r#"/[a](b)[[c]]/v.test('abc')"#,
        r#"/[a]b([[c]])/v.test('abc')"#,
        r#"/(?:[a]b[[c]])+/v.test('abc')"#,
        r#"/[a]+b[[c]]/v.test('abc')"#,
        r#"/[a]b[[c]]?/v.test('abc')"#,
        r#"/[a]\bb[[c]]/v.test('abc')"#,
        r#"/[a]b\b[[c]]/v.test('abc')"#,
        r#"/\b^[a]b[[c]]/v.test('abc')"#,
        r#"/[a]b[[c]]$\b/v.test('abc')"#,
        r#"/\b\b[a]b[[c]]/v.test('abc')"#,
        r#"/[a]b[[c]]\b\b/v.test('abc')"#,
        r#"/[\q{a}]b[a--a]/v.test('')"#,
        r#"/[]b[\p{RGI_Emoji}&&c]/v.test('')"#,
        r#"/[]b[\q{c}&&c]/v.test('')"#,
        r#"/[\p{RGI_Emoji}]b[a--a]/v.test('')"#,
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
        .eval(r#"var kept=/(?<whole>([a]a[[a]]\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([a]a[[a]]\\b))'&&kept.lastIndex===1&&kept.exec('aaaa!').groups.whole==='aaa'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[a]a[[c]]/vg,marker=0"#).unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn flat_v_set_literal_checked_v_set_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete JSON and original UTF16 spans were hand-derived before Runtime source.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([a]b[[c]]))"#,
            JsString::from_code_units(vec![97, 97, 98, 99]),
            "dvg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":1,"input":"aabc","groups":{"whole":"abc"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a]b[[c]]))"}"#,
        ),
        (
            r#"(?<whole>([a]a[[b]]))"#,
            JsString::from_code_units(vec![97, 97, 97, 98, 33]),
            "dvg",
            0,
            r#"{"matches":["aab","aab","aab"],"index":1,"input":"aaab!","groups":{"whole":"aab"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a]a[[b]]))"}"#,
        ),
        (
            r#"(?<whole>([a]a[[a]]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvg",
            0,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a]a[[a]]\\b))"}"#,
        ),
        (
            r#"(?<whole>([a]a[[a]]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([a]a[[a]]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            1,
            r#"{"matches":["aaa","aaa","aaa"],"index":1,"input":"aaaa!","groups":{"whole":"aaa"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([a]a[[a]]\\b))"}"#,
        ),
        (
            r#"(?<whole>(\b[a]b[[c]]))"#,
            JsString::from_code_units(vec![97, 98, 99, 120]),
            "dvg",
            0,
            r#"{"matches":["abc","abc","abc"],"index":0,"input":"abcx","groups":{"whole":"abc"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"(?<whole>(\\b[a]b[[c]]))"}"#,
        ),
        (
            r#"(?<whole>(\b[\u{1f600}]\u{1f601}[[\u{1f600}]]\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56833, 55357, 56832, 33]),
            "dvy",
            2,
            r#"{"matches":["😀😁😀","😀😁😀","😀😁😀"],"index":1,"input":"A😀😁😀!","groups":{"whole":"😀😁😀"},"indices":[[1,7],[1,7],[1,7]],"indexGroups":{"whole":[1,7]},"lastIndex":7,"source":"(?<whole>(\\b[\\u{1f600}]\\u{1f601}[[\\u{1f600}]]\\B))"}"#,
        ),
        (
            r#"(?<whole>(\b[\uD800]\uD800[[\uD800]]\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
            r#"{"matches":["\ud800\ud800\ud800","\ud800\ud800\ud800","\ud800\ud800\ud800"],"index":1,"input":"A\ud800\ud800\ud800!","groups":{"whole":"\ud800\ud800\ud800"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>(\\b[\\uD800]\\uD800[[\\uD800]]\\B))"}"#,
        ),
        (
            r#"^(?<whole>(\B[\p{Any}]\$[[\p{Any}]]\b))$"#,
            JsString::from_code_units(vec![33, 36, 8490]),
            "divg",
            0,
            r#"{"matches":["!$K","!$K","!$K"],"index":0,"input":"!$K","groups":{"whole":"!$K"},"indices":[[0,3],[0,3],[0,3]],"indexGroups":{"whole":[0,3]},"lastIndex":3,"source":"^(?<whole>(\\B[\\p{Any}]\\$[[\\p{Any}]]\\b))$"}"#,
        ),
        (
            r#"(?<whole>([]c[[\p{Any}]]))"#,
            JsString::from_code_units(vec![97, 99, 98, 33]),
            "dvg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
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
fn single_character_v_strings_state_context_and_consumers() {
    check(r#"/a[\q{b}]/v.test('ab')"#);
    check(r#"/[\q{a|b}]/v.test('a')&&/[\q{a|b}]/v.test('b')&&!/[\q{a|b}]/v.test('c')"#);
    check(
        r#"! /[^\q{a|b}]/v.test('ab')&&/[^\q{a|b}]/v.test('c')&&/[^\q{a|b}]/v.exec('😀')[0]==='😀'"#,
    );
    check(
        r#"let r=/[\q{K|s}]/iv;r.test('K')&&r.test('ſ')&&r.test('k')&&!r.test('!')&&!/[\q{K|s}]/v.test('K')"#,
    );
    check(r#"! /[^\q{K|s}]/iv.test('Kſ')&&/[^\q{K|s}]/iv.test('!')&&/[^\q{K|s}]/v.test('K')"#);
    check(
        r#"/^\b[\q{\u212A}]\b$/iv.test('K')&&!/^\b[\q{\u212A}]\b$/v.test('K')&&/^[\q{\u212A}]$/v.test('K')"#,
    );
    check(
        r#"/[\q{a}]/v.test('a')&&/\b[\q{a}]/v.test('a')&&/[\q{a}]\b/v.test('a')&&/\b[\q{a}]\b/v.test('a')&&/\B[\q{a}]/v.test('xa')&&/\B[\q{a}]\b/v.test('xa!')&&/\b[\q{a}]\B/v.test('ax')&&/\B[\q{a}]\B/v.test('xax')"#,
    );
    check(
        r#"let r=/^\B[\q{\r|\n|\u2028}]\B$/mv,m=r.exec('x\r\n\u2028y');m!==null&&m.index===2&&m[0]==='\n'&&!/^\B[\q{\r|\n|\u2028}]\B$/v.test('x\r\n\u2028y')"#,
    );
    check(
        r#"let r=/\b[\q{\uD83D\uDE00|x}]\B/dvy;r.lastIndex=2;let m=r.exec('A😀!');m!==null&&m.index===1&&m[0]==='😀'&&m.indices[0][0]===1&&m.indices[0][1]===3&&r.lastIndex===3"#,
    );
    check(
        r#"let r=/[\q{a}]/vy;r.lastIndex=0;let a=r.exec('xa');r.lastIndex=1;let b=r.exec('xa');a===null&&b[0]==='a'&&b.index===1&&r.lastIndex===2"#,
    );
    check(
        r#"/[\q{😀|x}]/v.exec('😀')[0]==='😀'&&/[\q{\uD83D\uDE00|x}]/v.exec('😀')[0]==='😀'&&/[\q{\u{1f600}|x}]/v.exec('😀')[0]==='😀'"#,
    );
    check(
        r#"/[\q{\uD800|\uDC00}]/v.exec('A\uD800B').index===1&&/[\q{\u{D800}}]/v.exec('\uD800')[0]==='\uD800'&&!/[\q{\uD800|\uDC00}]/v.test('😀')"#,
    );
    check(
        r#"let r=/[\q{\b|\!|\||\}|\/}]/v;r.test('\b')&&r.test('!')&&r.test('|')&&r.test('}')&&r.test('/')&&!r.test('b')"#,
    );
    check(
        r#"let r=/[\q{\cA|\0|\x61|\u0062}]/v;r.test('\x01')&&r.test('\0')&&r.test('a')&&r.test('b')&&!r.test('c')"#,
    );
    check(
        r#"let r=/[\q{a|a|b|a}]/dgv;r.flags==='dgv'&&r.unicodeSets&&!r.unicode&&r.exec('b')[0]==='b'"#,
    );
    check(
        r#"let r=/(?<whole>([\q{a|b}]))/dgv,m=r.exec('za!');Array.isArray(m)&&m.length===3&&m[0]==='a'&&m[1]==='a'&&m[2]==='a'&&m.index===1&&m.input==='za!'&&m.groups.whole==='a'&&m.indices[0][0]===1&&m.indices[0][1]===2&&m.indices.groups.whole[0]===1&&m.indices.groups.whole[1]===2&&r.lastIndex===2"#,
    );
    check(
        r#"let trace='',r=/[\q{a|b}]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return 'xa'}});trace==='si'&&m[0]==='a'&&m.index===1&&r.lastIndex===2"#,
    );
    check(
        r#"let seen=0,r=/[\q{a|b}]/v;r.lastIndex={valueOf(){seen++;return 3}};r.exec('za').index===1&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[\q{K|s}]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.source==='[\\q{K|s}]'&&c.flags==='dvy'&&c.lastIndex===0&&c.exec('K')===null&&r.exec('K')[0]==='K'&&r.lastIndex===7"#,
    );
    check(
        r#"let a=/[\q{a}]/vy,b=/[\q{a}]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('xa')}catch(e){x=e instanceof TypeError}try{b.exec('x')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>([\q{a|b}]))/dgv,a=[...'a-b!'.matchAll(r)];a.length===2&&a[0].index===0&&a[1].index===2&&a[1].groups.whole==='b'&&'a-b!'.replace(r,'<$<whole>>')==='<a>-<b>!'&&'xa-bz'.split(/[\q{a|b}]/v).join('|')==='x|-|z'"#,
    );
    check(
        r#"let r=/[\q{a|b}]/vg;r.lastIndex=1;'za'.search(r)===1&&r.lastIndex===1&&'ab'.match(r).join(',')==='a,b'&&r.lastIndex===0"#,
    );
    for pattern in [
        r#"[^\q{}]"#,
        r#"[^\q{a|ab}]"#,
        r#"[\q{\d}]"#,
        r#"[\q{\_}]"#,
        r#"[\q{a]"#,
    ] {
        check(&format!(
            "let caught=false;try{{new RegExp({pattern:?},'v')}}catch(e){{caught=e instanceof SyntaxError}}caught"
        ));
    }
    for (source, expected) in [
        (r#"/[\q{a}b]/v.test('a')"#, true),
        (r#"/[[\q{a}]]/v.test('a')"#, true),
        (r#"/[\q{a}&&a]/v.test('a')"#, true),
        (r#"/[^\q{a}&&a]/v.test('a')"#, false),
        (r#"/[\q{a}\q{b}]/v.test('a')"#, true),
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Boolean(expected)),
            "{source}"
        );
    }
    check(r#"/[\q{a|b}]+/v.test('ab')"#);
    assert_eq!(
        Realm::default().eval(r#"/[\q{a}]b/v.test('ab')"#),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        Realm::default().eval(r#"/[\q{a}][b]/v.test('ab')"#),
        Ok(Value::Boolean(true))
    );
    for (source, expected) in [
        (r#"/[\q{}]/v.test('')"#, true),
        (r#"/[\q{a|}]/v.test('a')"#, true),
        (r#"/[\q{|a}]/v.test('a')"#, true),
        (r#"/[\q{a|b|ab}]/v.test('a')"#, true),
        (r#"/[\q{ab|c}]/v.test('ab')"#, true),
        (r#"/[\q{\u{D800}\u{DC00}}]/v.test('\uD800\uDC00')"#, false),
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Boolean(expected)),
            "{source}"
        );
    }
    for source in [
        r#"/([\q{a}])b/v.test('ab')"#,
        r#"/\b^[\q{a}]/v.test('a')"#,
        r#"/[\q{a}]$\b/v.test('a')"#,
        r#"/[\p{RGI_Emoji}]/v.test('😀')"#,
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
        .eval(r#"var kept=/(?<whole>([\q{a|b}]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([\\q{a|b}]))'&&kept.lastIndex===1&&kept.exec('za').groups.whole==='a'&&kept.lastIndex===2"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[\q{z}]/vg,marker=0"#).unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn single_character_v_strings_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete JSON/original UTF16 spans were hand-derived before Runtime source.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([\q{a|b}]))"#,
            JsString::from_code_units(vec![122, 98]),
            "dvg",
            0,
            r#"{"matches":["b","b","b"],"index":1,"input":"zb","groups":{"whole":"b"},"indices":[[1,2],[1,2],[1,2]],"indexGroups":{"whole":[1,2]},"lastIndex":2,"source":"(?<whole>([\\q{a|b}]))"}"#,
        ),
        (
            r#"(?<whole>([^\q{a|b}]))"#,
            JsString::from_code_units(vec![97, 98, 55357, 56832]),
            "dvg",
            0,
            r#"{"matches":["😀","😀","😀"],"index":2,"input":"ab😀","groups":{"whole":"😀"},"indices":[[2,4],[2,4],[2,4]],"indexGroups":{"whole":[2,4]},"lastIndex":4,"source":"(?<whole>([^\\q{a|b}]))"}"#,
        ),
        (
            r#"(?<whole>([\q{K|s}]))"#,
            JsString::from_code_units(vec![8490]),
            "divg",
            0,
            r#"{"matches":["K","K","K"],"index":0,"input":"K","groups":{"whole":"K"},"indices":[[0,1],[0,1],[0,1]],"indexGroups":{"whole":[0,1]},"lastIndex":1,"source":"(?<whole>([\\q{K|s}]))"}"#,
        ),
        (
            r#"(?<whole>([^\q{K|s}]))"#,
            JsString::from_code_units(vec![8490, 383]),
            "divg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"^(?<whole>(\b[\q{\u212A}]\b))$"#,
            JsString::from_code_units(vec![8490]),
            "divg",
            0,
            r#"{"matches":["K","K","K"],"index":0,"input":"K","groups":{"whole":"K"},"indices":[[0,1],[0,1],[0,1]],"indexGroups":{"whole":[0,1]},"lastIndex":1,"source":"^(?<whole>(\\b[\\q{\\u212A}]\\b))$"}"#,
        ),
        (
            r#"^(?<whole>(\b[\q{\u212A}]\b))$"#,
            JsString::from_code_units(vec![8490]),
            "dvg",
            0,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>(\b[\q{\uD83D\uDE00|x}]\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 33]),
            "dvy",
            2,
            r#"{"matches":["😀","😀","😀"],"index":1,"input":"A😀!","groups":{"whole":"😀"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\b[\\q{\\uD83D\\uDE00|x}]\\B))"}"#,
        ),
        (
            r#"(?<whole>([\q{\uD800|\uDC00}]))"#,
            JsString::from_code_units(vec![65, 55296, 66]),
            "dvg",
            0,
            r#"{"matches":["\ud800","\ud800","\ud800"],"index":1,"input":"A\ud800B","groups":{"whole":"\ud800"},"indices":[[1,2],[1,2],[1,2]],"indexGroups":{"whole":[1,2]},"lastIndex":2,"source":"(?<whole>([\\q{\\uD800|\\uDC00}]))"}"#,
        ),
        (
            r#"(?<whole>([\q{\b|\!|\||\}|\/}]))"#,
            JsString::from_code_units(vec![120, 8]),
            "dvg",
            0,
            r#"{"matches":["\b","\b","\b"],"index":1,"input":"x\b","groups":{"whole":"\b"},"indices":[[1,2],[1,2],[1,2]],"indexGroups":{"whole":[1,2]},"lastIndex":2,"source":"(?<whole>([\\q{\\b|\\!|\\||\\}|\\/}]))"}"#,
        ),
        (
            r#"^(?<whole>(\B[\q{\r|\n|\u2028}]\B))$"#,
            JsString::from_code_units(vec![120, 13, 10, 8232, 121]),
            "dgmv",
            0,
            r#"{"matches":["\n","\n","\n"],"index":2,"input":"x\r\n y","groups":{"whole":"\n"},"indices":[[2,3],[2,3],[2,3]],"indexGroups":{"whole":[2,3]},"lastIndex":3,"source":"^(?<whole>(\\B[\\q{\\r|\\n|\\u2028}]\\B))$"}"#,
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
fn single_character_v_string_operands_state_context_and_consumers() {
    check(r#"/[a\q{b|c}]/v.test('a')&&/[a\q{b|c}]/v.test('b')&&!/[a\q{b|c}]/v.test('z')"#);
    check(
        r#"/[a[\q{b|c}]d-f]/v.test('e')&&/[a[\q{b|c}]d-f]/v.test('c')&&!/[a[\q{b|c}]d-f]/v.test('x')"#,
    );
    check(r#"/[\q{a|b}\d]/v.test('7')&&/[\q{a|b}\d]/v.test('b')&&!/[\q{a|b}\d]/v.test('z')"#);
    check(r#"/[\q{a|b}&&[b-z]]/v.exec('aab').index===2&&!/[\q{a|b}&&[b-z]]/v.test('a')"#);
    check(
        r#"/[\q{a|b}--b]/v.test('a')&&!/[\q{a|b}--b]/v.test('b')&&!/[a--\q{a|b}]/v.test('a')&&/[[a-c]--\q{b}]/v.test('c')"#,
    );
    check(
        r#"/[\q{a|b}&&[^\q{a}]]/v.test('b')&&!/[\q{a|b}&&[^\q{a}]]/v.test('a')&&!/[^\q{a}&&a]/v.test('a')&&/[^\q{a}&&a]/v.test('b')"#,
    );
    check(
        r#"/[\q{a}\q{b}]/v.test('b')&&/[\q{a|b}&&\q{b|c}]/v.test('b')&&!/[\q{a|b}&&\q{b|c}]/v.test('a')"#,
    );
    check(r#"/[\q{a|😀}&&\p{ASCII}]/v.test('a')&&!/[\q{a|😀}&&\p{ASCII}]/v.test('😀')"#);
    check(
        r#"/[\q{K|s}--k]/iv.test('ſ')&&!/[\q{K|s}--k]/iv.test('KKk')&&/[\q{K|s}--k]/v.test('K')"#,
    );
    check(r#"! /[^[[\q{K|s}]--[s]]]/iv.test('KKk')&&/[^[[\q{K|s}]--[s]]]/iv.test('ſ!')"#);
    check(r#"/[\q{Σ|µ}--ς]/iv.test('μ')&&!/[\q{Σ|µ}--ς]/iv.test('Σςσ')"#);
    check(
        r#"let r=/[\q{\b|\!|\||\}|\/}&&[^\q{!}]]/v;r.test('\b')&&r.test('|')&&r.test('}')&&r.test('/')&&!r.test('!')&&!r.test('b')"#,
    );
    check(
        r#"/[\q{\]|\[|\}}x]/v.test(']')&&/[\q{\]|\[|\}}x]/v.test('[')&&/[\q{\]|\[|\}}x]/v.test('}')&&/[\q{\]|\[|\}}x]/v.test('x')"#,
    );
    check(
        r#"/^\b[\q{K}&&k]\b$/ivm.test('x\nK\nz')&&!/^\b[\q{K}&&k]\b$/vm.test('x\nK\nz')&&!/^\b[\q{K}&&k]\b$/iv.test('x\nK\nz')"#,
    );
    check(
        r#"let r=/\b[\q{\uD83D\uDE00|x}&&[^\q{x}]]\B/dvy;r.lastIndex=2;let m=r.exec('A😀!');m.index===1&&m[0]==='😀'&&m.indices[0][0]===1&&m.indices[0][1]===3&&r.lastIndex===3"#,
    );
    check(
        r#"/[\q{\uD800|\uDC00}--[\q{\uDC00}]]/v.exec('A\uD800B').index===1&&!/[\q{\uD800|\uDC00}--[\q{\uDC00}]]/v.test('😀')"#,
    );
    check(
        r#"let trace='',r=/[\q{a|b}&&[b]]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return 'xb'}});trace==='si'&&m[0]==='b'&&m.index===1&&r.lastIndex===2"#,
    );
    check(
        r#"let seen=0,r=/[\q{a|b}&&[b]]/v;r.lastIndex={valueOf(){seen++;return 3}};r.exec('xb').index===1&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[\q{K|s}--k]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.source==='[\\q{K|s}--k]'&&c.flags==='dvy'&&c.lastIndex===0&&c.exec('ſ')===null&&r.exec('ſ')[0]==='ſ'&&r.lastIndex===7"#,
    );
    check(
        r#"let a=/[\q{a|b}&&[b]]/vy,b=/[\q{a|b}&&[b]]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('xb')}catch(e){x=e instanceof TypeError}try{b.exec('x')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>([\q{a|b}&&[b]]))/dvg,a=[...'a-bb!'.matchAll(r)];a.length===2&&a[0].index===2&&a[1].index===3&&a[1].groups.whole==='b'&&'a-bb!'.replace(r,'<$<whole>>')==='a-<b><b>!'&&'a-bb!'.split(/[\q{a|b}&&[b]]/v).join('|')==='a-||!'"#,
    );
    check(
        r#"let r=/[\q{a|b}&&[b]]/vg;r.lastIndex=1;'xab'.search(r)===2&&r.lastIndex===1&&'abbb'.match(r).join(',')==='b,b,b'&&r.lastIndex===0"#,
    );
    for pattern in [
        r"[^\q{a|ab}]",
        r"[\q{a}-b]",
        r"[a-\q{b}]",
        r"[\q{a}&&a--a]",
        r"[\q{\d}&&a]",
        r"[\q{!!}a]",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({code:?})}}catch(e){{caught=e instanceof SyntaxError}}caught&&marker===0"
        ));
    }
    for (source, expected) in [
        (r"/[^\q{ab}&&a]/v.test('a')", true),
        (r"/[[a--a]&&[\q{ab}]]/v.test('a')", false),
        (r"/[\q{a|b}]+/v.test('ab')", true),
        (r"/[[\q{a|b}]]{0}/v.test('')", true),
        (r"/a[\q{b}]/v.test('ab')", true),
        (r"/[a--a--\q{ab}]/v.test('a')", false),
        (r"/[\q{}&&a]/v.test('a')", false),
        (r"/[[\q{a|b}]--[\q{a|bc}]]/v.test('b')", true),
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(expected)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    {
        let source = r"/[\q{a}&&a]b/v.test('ab')";
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(true)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    for source in [
        r"/[[\q{a|b}]&&[\p{RGI_Emoji}]]/v.test('a')",
        r"/\b^[\q{a}&&a]/v.test('a')",
        r"/[\q{a}&&a]$\b/v.test('a')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)), "{source}");
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>([\q{a|b}&&[b]]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([\\q{a|b}&&[b]]))'&&kept.lastIndex===1&&kept.exec('zb').groups.whole==='b'&&kept.lastIndex===2"#),Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[\q{z}&&z]/vg,marker=0"#).unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn single_character_v_string_operands_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full JSON and original UTF16 spans were fixed before this Runtime source.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([a\q{b|c}]))"#,
            JsString::from_code_units(vec![122, 98]),
            "dvg",
            0usize,
            r#"{"matches":["b","b","b"],"index":1,"input":"zb","groups":{"whole":"b"},"indices":[[1,2],[1,2],[1,2]],"indexGroups":{"whole":[1,2]},"lastIndex":2,"source":"(?<whole>([a\\q{b|c}]))"}"#,
        ),
        (
            r#"(?<whole>([\q{a|b}&&[b-z]]))"#,
            JsString::from_code_units(vec![97, 97, 98]),
            "dvg",
            0usize,
            r#"{"matches":["b","b","b"],"index":2,"input":"aab","groups":{"whole":"b"},"indices":[[2,3],[2,3],[2,3]],"indexGroups":{"whole":[2,3]},"lastIndex":3,"source":"(?<whole>([\\q{a|b}&&[b-z]]))"}"#,
        ),
        (
            r#"(?<whole>([\q{K|s}--k]))"#,
            JsString::from_code_units(vec![8490, 115]),
            "divg",
            0usize,
            r#"{"matches":["s","s","s"],"index":1,"input":"Ks","groups":{"whole":"s"},"indices":[[1,2],[1,2],[1,2]],"indexGroups":{"whole":[1,2]},"lastIndex":2,"source":"(?<whole>([\\q{K|s}--k]))"}"#,
        ),
        (
            r#"(?<whole>([^[[\q{K|s}]--[s]]]))"#,
            JsString::from_code_units(vec![8490]),
            "divg",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"^(?<whole>(\b[\q{K}&&k]\b))$"#,
            JsString::from_code_units(vec![120, 10, 8490, 10, 122]),
            "dgimv",
            0usize,
            r#"{"matches":["K","K","K"],"index":2,"input":"x\nK\nz","groups":{"whole":"K"},"indices":[[2,3],[2,3],[2,3]],"indexGroups":{"whole":[2,3]},"lastIndex":3,"source":"^(?<whole>(\\b[\\q{K}&&k]\\b))$"}"#,
        ),
        (
            r#"^(?<whole>(\b[\q{K}&&k]\b))$"#,
            JsString::from_code_units(vec![120, 10, 8490, 10, 122]),
            "dgmv",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>(\b[\q{\uD83D\uDE00|x}&&[^\q{x}]]\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 33]),
            "dvy",
            2usize,
            r#"{"matches":["😀","😀","😀"],"index":1,"input":"A😀!","groups":{"whole":"😀"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\b[\\q{\\uD83D\\uDE00|x}&&[^\\q{x}]]\\B))"}"#,
        ),
        (
            r#"(?<whole>([\q{\uD800|\uDC00}--[\q{\uDC00}]]))"#,
            JsString::from_code_units(vec![65, 55296, 66]),
            "dvg",
            0usize,
            r#"{"matches":["\ud800","\ud800","\ud800"],"index":1,"input":"A\ud800B","groups":{"whole":"\ud800"},"indices":[[1,2],[1,2],[1,2]],"indexGroups":{"whole":[1,2]},"lastIndex":2,"source":"(?<whole>([\\q{\\uD800|\\uDC00}--[\\q{\\uDC00}]]))"}"#,
        ),
        (
            r#"(?<whole>([\q{\b|\!|\||\}|\/}&&[^\q{!}]]))"#,
            JsString::from_code_units(vec![33, 124]),
            "dvg",
            0usize,
            r#"{"matches":["|","|","|"],"index":1,"input":"!|","groups":{"whole":"|"},"indices":[[1,2],[1,2],[1,2]],"indexGroups":{"whole":[1,2]},"lastIndex":2,"source":"(?<whole>([\\q{\\b|\\!|\\||\\}|\\/}&&[^\\q{!}]]))"}"#,
        ),
        (
            r#"^(?<whole>(\B[[\q{\r|\n|\u2028}]--[\q{\r}]]\B))$"#,
            JsString::from_code_units(vec![120, 13, 10, 8232, 121]),
            "dgmv",
            0usize,
            r#"{"matches":["\n","\n","\n"],"index":2,"input":"x\r\n y","groups":{"whole":"\n"},"indices":[[2,3],[2,3],[2,3]],"indexGroups":{"whole":[2,3]},"lastIndex":3,"source":"^(?<whole>(\\B[[\\q{\\r|\\n|\\u2028}]--[\\q{\\r}]]\\B))$"}"#,
        ),
    ] {
        let script = format!(
            "let r=new RegExp({pattern:?},{flags:?});r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:Array.from(m),index:m.index,input:m.input,groups:m.groups,indices:Array.from(m.indices),indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
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
fn single_character_v_string_repetition_state_context_and_consumers() {
    check(
        r#"let a=/[\q{a|b}]+/v.exec('zabba!'),b=/[\q{a|b}]+?/v.exec('zabba!');a.index===1&&a[0]==='abba'&&b.index===1&&b[0]==='a'"#,
    );
    check(
        r#"/^[a\q{b}]*$/v.test('abba')&&/^[a\q{b}]?$/v.test('')&&/^[a\q{b}]?$/v.test('b')&&!/^[a\q{b}]?$/v.test('bb')"#,
    );
    check(
        r#"let r=/[\q{a}]{1,2}\b/v,m=r.exec('aaa');m.index===1&&m[0]==='aa'&&/[\q{a}]{1,2}?\b/v.exec('aaa')[0]==='aa'"#,
    );
    check(
        r#"let r=/[\q{a}]{1,2}\b/vy;r.lastIndex=0;let a=r.exec('aaa');r.lastIndex=1;let b=r.exec('aaa');a===null&&b.index===1&&b[0]==='aa'&&r.lastIndex===3"#,
    );
    check(
        r#"let r=/\b[\q{a}--a]{0}\b/dvg,m=r.exec('a');m.index===0&&m[0]===''&&m.indices[0][0]===0&&m.indices[0][1]===0&&r.lastIndex===0&&/\B[\q{a}--a]{0}\B/v.test('')"#,
    );
    check(
        r#"/[\q{a|b}]{0001,0003}/v.exec('abbb!')[0]==='abb'&&/[\q{a|b}]{2,}/v.exec('abbb!')[0]==='abbb'&&!/[\q{a}]{999999999999999999999999}/v.test('a')&&/[\q{a}]{1,999999999999999999999999}/v.exec('aaa!')[0]==='aaa'"#,
    );
    check(r#"/^[\q{a}&&[]]*$/v.test('')&&!/^[\q{a}&&[]]*$/v.test('a')&&!/[\q{a}&&[]]+/v.test('')"#);
    check(r#"/[\q{K}&&k]+/iv.exec('!KKk!')[0]==='KKk'&&!/[\q{K}&&k]+/v.test('KkK')"#);
    check(r#"let r=/[^\q{K|s}]+?/iv,m=r.exec('Kſ!');m.index===2&&m[0]==='!'&&!r.test('Kſ')"#);
    check(
        r#"/^\b[\q{K}&&k]+\b$/ivm.exec('x\nKKk\nz')[0]==='KKk'&&!/^\b[\q{K}&&k]+\b$/iv.test('x\nKKk\nz')"#,
    );
    check(
        r#"let m=/^\B[[\q{\r|\n|\u2028}]--[\q{\r}]]{1,2}\B$/mv.exec('x\r\n\u2028y');m.index===2&&m[0]==='\n'"#,
    );
    check(
        r#"/[\q{\{|\}|\?}]{1,2}?/v.exec('{}?')[0]==='{'&&/[\q{\{|\}|\?}]{1,2}/v.exec('{}?')[0]==='{}'"#,
    );
    check(
        r#"let r=/(?<whole>(\b[\q{\uD83D\uDE00|x}&&[^\q{x}]]{1,2}\B))/dvy;r.lastIndex=2;let m=r.exec('A😀😀!');m.index===1&&m[0]==='😀😀'&&m.groups.whole==='😀😀'&&m.indices[0][0]===1&&m.indices[0][1]===5&&r.lastIndex===5"#,
    );
    check(
        r#"let m=/[\q{\uD800|x}--x]{2}/v.exec('A\uD800\uD800B');m.index===1&&m[0].length===2&&m[0].charCodeAt(0)===0xD800&&m[0].charCodeAt(1)===0xD800"#,
    );
    check(
        r#"let r=/(?<whole>([\q{a|b}]+))/dvg,m=r.exec('zabba!');m.length===3&&m[0]==='abba'&&m[1]==='abba'&&m[2]==='abba'&&m.groups.whole==='abba'&&m.indices[0][0]===1&&m.indices[0][1]===5&&m.indices.groups.whole[1]===5&&r.lastIndex===5"#,
    );
    check(
        r#"let trace='',r=/[\q{a|b}]+/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return 'zabb'}});trace==='si'&&m[0]==='abb'&&m.index===1&&r.lastIndex===4"#,
    );
    check(
        r#"let seen=0,r=/[\q{a|b}]+/v;r.lastIndex={valueOf(){seen++;return 3}};r.exec('zab').index===1&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[\q{K}]+/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.source==='[\\q{K}]+'&&c.flags==='dvy'&&c.lastIndex===0&&c.exec('K')===null&&r.exec('K')[0]==='K'&&r.lastIndex===7"#,
    );
    check(
        r#"let a=/[\q{a}]+/vy,b=/[\q{a}]+/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('za')}catch(e){x=e instanceof TypeError}try{b.exec('z')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>([\q{a|b}]+))/dvg,a=[...'a-bb!'.matchAll(r)];a.length===2&&a[0].index===0&&a[1].index===2&&a[1].groups.whole==='bb'&&'a-bb!'.replace(r,'<$<whole>>')==='<a>-<bb>!'&&'xa-bbz'.split(/[\q{a|b}]+/v).join('|')==='x|-|z'"#,
    );
    check(
        r#"let r=/[\q{a|b}]+/vg;r.lastIndex=1;'zaab'.search(r)===1&&r.lastIndex===1&&'ab-ab'.match(r).join('|')==='ab|ab'&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[\q{a}--a]*/vg,a=[...'A😀B'.matchAll(r)];a.map(m=>m.index).join(',')==='0,1,3,4'&&r.lastIndex===0&&'A😀B'.replace(r,'|')==='|A|😀|B|'"#,
    );
    for pattern in [
        r"[\q{a}]{3,2}",
        r"[\q{a}]{,2}",
        r"[\q{a}]{1,2,3}",
        r"[^\q{ab}]{0}",
        r"[\q{\d}]+",
        r"[\q{a}-b]+",
        r"[\q{a}&&a--b]+",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({code:?})}}catch(e){{caught=e instanceof SyntaxError}}caught&&marker===0"
        ));
    }
    for source in [
        r"/([\q{a}])+/v.test('a')",
        r"/(?:[\q{a}])+/v.test('a')",
        r"/[^\q{ab}&&a]{0}/v.test('')",
        r"/[\q{a}&&[]&&\q{b|cd}]{0}/v.test('')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(true)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    for source in [
        r"/[\q{}]{0}/v.test('')",
        r"/[\q{a|}]*/v.test('a')",
        r"/[\q{a|bc}]{0}/v.test('')",
        r"/[\q{a}--a--\p{RGI_Emoji}]{0}/v.test('')",
        r"/[\q{a|bc}]{999999999999999999999999}/v.test('a')",
        r"/[\q{a}]+b/v.test('ab')",
        r"/a[\q{b}]+/v.test('ab')",
        r"/\b^[\q{a}]+/v.test('a')",
        r"/[\q{a}]+$\b/v.test('a')",
        r"/[\q{a}]+[b]+/v.test('ab')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)), "{source}");
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>([\q{a|b}&&[b]]+))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([\\q{a|b}&&[b]]+))'&&kept.lastIndex===1&&kept.exec('zbbb!').groups.whole==='bbb'&&kept.lastIndex===4"#),Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[\q{z}&&z]+/vg,marker=0"#).unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn single_character_v_string_repetition_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete JSON and original UTF16 spans were frozen before Runtime source.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([\q{a|b}]+))"#,
            JsString::from_code_units(vec![122, 97, 98, 98, 97, 33]),
            "dvg",
            0usize,
            r#"{"matches":["abba","abba","abba"],"index":1,"input":"zabba!","groups":{"whole":"abba"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>([\\q{a|b}]+))"}"#,
        ),
        (
            r#"(?<whole>([\q{a|b}&&[b-z]]{1,3}?))"#,
            JsString::from_code_units(vec![97, 98, 98, 98, 33]),
            "dvg",
            0usize,
            r#"{"matches":["b","b","b"],"index":1,"input":"abbb!","groups":{"whole":"b"},"indices":[[1,2],[1,2],[1,2]],"indexGroups":{"whole":[1,2]},"lastIndex":2,"source":"(?<whole>([\\q{a|b}&&[b-z]]{1,3}?))"}"#,
        ),
        (
            r#"(?<whole>(\b[\q{a}--a]{0}\b))"#,
            JsString::from_code_units(vec![97]),
            "dvg",
            0usize,
            r#"{"matches":["","",""],"index":0,"input":"a","groups":{"whole":""},"indices":[[0,0],[0,0],[0,0]],"indexGroups":{"whole":[0,0]},"lastIndex":0,"source":"(?<whole>(\\b[\\q{a}--a]{0}\\b))"}"#,
        ),
        (
            r#"(?<whole>([\q{a}]{999999999999999999999999}))"#,
            JsString::from_code_units(vec![97]),
            "dvg",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([\q{K}&&k]+))"#,
            JsString::from_code_units(vec![33, 8490, 75, 107, 33]),
            "divg",
            0usize,
            r#"{"matches":["KKk","KKk","KKk"],"index":1,"input":"!KKk!","groups":{"whole":"KKk"},"indices":[[1,4],[1,4],[1,4]],"indexGroups":{"whole":[1,4]},"lastIndex":4,"source":"(?<whole>([\\q{K}&&k]+))"}"#,
        ),
        (
            r#"(?<whole>([^\q{K|s}]+?))"#,
            JsString::from_code_units(vec![8490, 383, 33]),
            "divg",
            0usize,
            r#"{"matches":["!","!","!"],"index":2,"input":"Kſ!","groups":{"whole":"!"},"indices":[[2,3],[2,3],[2,3]],"indexGroups":{"whole":[2,3]},"lastIndex":3,"source":"(?<whole>([^\\q{K|s}]+?))"}"#,
        ),
        (
            r#"(?<whole>(\b[\q{\uD83D\uDE00|x}&&[^\q{x}]]{1,2}\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56832, 33]),
            "dvy",
            2usize,
            r#"{"matches":["😀😀","😀😀","😀😀"],"index":1,"input":"A😀😀!","groups":{"whole":"😀😀"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>(\\b[\\q{\\uD83D\\uDE00|x}&&[^\\q{x}]]{1,2}\\B))"}"#,
        ),
        (
            r#"(?<whole>([\q{\uD800|x}--x]{2}))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 66]),
            "dvg",
            0usize,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800B","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([\\q{\\uD800|x}--x]{2}))"}"#,
        ),
        (
            r#"^(?<whole>(\B[[\q{\r|\n|\u2028}]--[\q{\r}]]{1,2}\B))$"#,
            JsString::from_code_units(vec![120, 13, 10, 8232, 121]),
            "dgmv",
            0usize,
            r#"{"matches":["\n","\n","\n"],"index":2,"input":"x\r\n y","groups":{"whole":"\n"},"indices":[[2,3],[2,3],[2,3]],"indexGroups":{"whole":[2,3]},"lastIndex":3,"source":"^(?<whole>(\\B[[\\q{\\r|\\n|\\u2028}]--[\\q{\\r}]]{1,2}\\B))$"}"#,
        ),
        (
            r#"(?<whole>([\q{a|b}--a]{2,3}))"#,
            JsString::from_code_units(vec![97, 98, 98, 33]),
            "dvy",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
    ] {
        let script = format!(
            "let r=new RegExp({pattern:?},{flags:?});r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:Array.from(m),index:m.index,input:m.input,groups:m.groups,indices:Array.from(m.indices),indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
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
fn literal_single_character_v_string_class_state_context_and_consumers() {
    check(r#"/a[\q{b}]/v.exec('aab!').index===1&&!/a[\q{b}]/v.test('ax')&&!/a[\q{b}]/v.test('a')"#);
    check(
        r#"/a[a\q{b|c}]/v.test('ac')&&/a[[\q{b|c}]d-f]/v.test('ae')&&/a[\q{a|b}&&[b-z]]/v.test('ab')&&!/a[\q{a|b}--b]/v.test('ab')&&!/a[\q{a}--a]/v.test('aa')"#,
    );
    check(
        r#"let r=/\Ba[\q{a}]\b/v,m=r.exec('aaa aa!');m.index===1&&m[0]==='aa'&&/\ba[\q{b}]\b/v.exec('xab ab!').index===4"#,
    );
    check(
        r#"let r=/a[\q{b}]/dvy;r.lastIndex=0;let a=r.exec('aab');r.lastIndex=1;let b=r.exec('aab');a===null&&b.index===1&&b[0]==='ab'&&b.indices[0][1]===3&&r.lastIndex===3&&r.exec('aab')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/a[\q{b}]/vg,a=r.exec('abab!'),b=r.exec('abab!');a.index===0&&b.index===2&&r.lastIndex===4&&r.exec('abab!')===null&&r.lastIndex===0"#,
    );
    check(r#"/s[\q{K}&&k]/iv.exec('!ſK!')[0]==='ſK'&&!/s[\q{K}&&k]/v.test('ſK')"#);
    check(
        r#"let r=/K[^\q{K|s}]/iv,m=r.exec('KſK!');m.index===2&&m[0]==='K!'&&!/s[^\q{K|s}]/iv.test('ſK')"#,
    );
    check(
        r#"let r=/\b\u{1f600}[\q{\uD83D\uDE01|x}--x]\B/dvy;r.lastIndex=2;let m=r.exec('A😀😁!');m.index===1&&m[0]==='😀😁'&&m.indices[0][0]===1&&m.indices[0][1]===5&&r.lastIndex===5"#,
    );
    check(
        r#"let m=/\uD800[\q{\uD800|x}--x]/v.exec('A\uD800\uD800B');m.index===1&&m[0].length===2&&m[0].charCodeAt(0)===0xD800&&m[0].charCodeAt(1)===0xD800&&!/\uD800[\q{\uD800}]/v.test('😀')"#,
    );
    check(
        r#"let r=/^\B\r[\q{\n|\u2028}]\B$/vm,m=r.exec('x\n\r\n\rz');m.index===2&&m[0]==='\r\n'&&!/^\B\r[\q{\n|\u2028}]\B$/v.test('x\n\r\n\rz')"#,
    );
    check(r#"/^\B\$[\q{\]|\|}]\B$/v.test('$]')&&/^\B\$[\q{\]|\|}]\B$/v.test('$|')"#);
    check(
        r#"let trace='',r=/a[\q{b}]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return 'aab!'}});trace==='si'&&m[0]==='ab'&&m.index===1&&r.lastIndex===3"#,
    );
    check(
        r#"let seen=0,r=/a[\q{b}]/v;r.lastIndex={valueOf(){seen++;return 3}};r.exec('aab!').index===1&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/s[\q{K}&&k]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.source==='s[\\q{K}&&k]'&&c.flags==='dvy'&&c.lastIndex===0&&c.exec('ſK')===null&&r.exec('ſK')[0]==='ſK'&&r.lastIndex===7"#,
    );
    check(
        r#"let a=/a[\q{b}]/vy,b=/a[\q{b}]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aab!')}catch(e){x=e instanceof TypeError}try{b.exec('aab!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>(a[\q{b}]))/dvg,a=[...'aab-ab!'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===4&&a[1].groups.whole==='ab'&&a[1].indices[2][1]===6&&'aab-ab!'.replace(r,'<$<whole>>')==='a<ab>-<ab>!'&&'aab-ab!'.split(/a[\q{b}]/v).join('|')==='a|-|!'"#,
    );
    check(
        r#"let r=/a[\q{b}]/vg;r.lastIndex=4;'aab!'.search(r)===1&&r.lastIndex===4&&'ab-ab'.match(r).join('|')==='ab|ab'&&r.lastIndex===0"#,
    );
    for pattern in [
        r"a[\q{a}-b]",
        r"a[^\q{ab}]",
        r"a[\q{\d}]",
        r"a[\q{\_}]",
        r"\u{110000}[\q{b}]",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({code:?})}}catch(e){{caught=e instanceof SyntaxError}}caught&&marker===0"
        ));
    }
    {
        let source = r"/[\q{a}]b/v.test('ab')";
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(true)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    for source in [
        r"/a[\q{b}]c/v.test('abc')",
        r"/(a)[\q{b}]/v.test('ab')",
        r"/a([\q{b}])/v.test('ab')",
        r"/a[\q{b}]+/v.test('ab')",
        r"/a+[\q{b}]/v.test('ab')",
        r"/(a[\q{b}])+/v.test('ab')",
        r"/a[\q{}]/v.test('a')",
        r"/a[\q{a|bc}]/v.test('aa')",
        r"/a[\q{a}&&[]&&\q{bc}]/v.test('aa')",
        r"/a[\q{a}--a--\p{RGI_Emoji}]/v.test('aa')",
        r"/a[\q{b}]$$/v.test('ab')",
        r"/\b\ba[\q{b}]\b/v.test('ab')",
        r"/\ba[\q{b}]\b\b/v.test('ab')",
        r"/\b^a[\q{b}]/v.test('ab')",
        r"/a\b[\q{b}]/v.test('ab')",
        r"/a[\q{b}]$\b/v.test('ab')",
        r"/a[\q{b}]|c/v.test('ab')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)), "{source}");
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>(a[\q{a|b}&&b]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>(a[\\q{a|b}&&b]))'&&kept.lastIndex===1&&kept.exec('aab!').groups.whole==='ab'&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/a[\q{z}]/vg,marker=0"#).unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn literal_single_character_v_string_class_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete JSON and original UTF16 spans were frozen before Runtime source.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>(a[\q{b}]))"#,
            JsString::from_code_units(vec![97, 97, 98]),
            "dvg",
            0usize,
            r#"{"matches":["ab","ab","ab"],"index":1,"input":"aab","groups":{"whole":"ab"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(a[\\q{b}]))"}"#,
        ),
        (
            r#"(?<whole>(\Ba[\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 32, 97, 97, 33]),
            "dvg",
            0usize,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa aa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\Ba[\\q{a}]\\b))"}"#,
        ),
        (
            r#"(?<whole>(s[\q{K}&&k]))"#,
            JsString::from_code_units(vec![33, 383, 8490, 33]),
            "divg",
            0usize,
            r#"{"matches":["ſK","ſK","ſK"],"index":1,"input":"!ſK!","groups":{"whole":"ſK"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(s[\\q{K}&&k]))"}"#,
        ),
        (
            r#"(?<whole>(K[^\q{K|s}]))"#,
            JsString::from_code_units(vec![8490, 383, 75, 33]),
            "divg",
            0usize,
            r#"{"matches":["K!","K!","K!"],"index":2,"input":"KſK!","groups":{"whole":"K!"},"indices":[[2,4],[2,4],[2,4]],"indexGroups":{"whole":[2,4]},"lastIndex":4,"source":"(?<whole>(K[^\\q{K|s}]))"}"#,
        ),
        (
            r#"(?<whole>(\b\u{1f600}[\q{\uD83D\uDE01|x}--x]\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56833, 33]),
            "dvy",
            2usize,
            r#"{"matches":["😀😁","😀😁","😀😁"],"index":1,"input":"A😀😁!","groups":{"whole":"😀😁"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>(\\b\\u{1f600}[\\q{\\uD83D\\uDE01|x}--x]\\B))"}"#,
        ),
        (
            r#"(?<whole>(\uD800[\q{\uD800|x}--x]))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 66]),
            "dvg",
            0usize,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800\ud800"],"index":1,"input":"A\ud800\ud800B","groups":{"whole":"\ud800\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>(\\uD800[\\q{\\uD800|x}--x]))"}"#,
        ),
        (
            r#"^(?<whole>(\B\r[\q{\n|\u2028}]\B))$"#,
            JsString::from_code_units(vec![120, 10, 13, 10, 13, 122]),
            "dgmv",
            0usize,
            r#"{"matches":["\r\n","\r\n","\r\n"],"index":2,"input":"x\n\r\n\rz","groups":{"whole":"\r\n"},"indices":[[2,4],[2,4],[2,4]],"indexGroups":{"whole":[2,4]},"lastIndex":4,"source":"^(?<whole>(\\B\\r[\\q{\\n|\\u2028}]\\B))$"}"#,
        ),
        (
            r#"^(?<whole>(\B\$[\q{\]|\|}]\B))$"#,
            JsString::from_code_units(vec![36, 93]),
            "dvg",
            0usize,
            r#"{"matches":["$]","$]","$]"],"index":0,"input":"$]","groups":{"whole":"$]"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"^(?<whole>(\\B\\$[\\q{\\]|\\|}]\\B))$"}"#,
        ),
        (
            r#"(?<whole>(s[^\q{K|s}]))"#,
            JsString::from_code_units(vec![383, 8490]),
            "divg",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>(a[\q{b}]))"#,
            JsString::from_code_units(vec![97, 97, 98]),
            "dvy",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
    ] {
        let script = format!(
            "let r=new RegExp({pattern:?},{flags:?});r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:Array.from(m),index:m.index,input:m.input,groups:m.groups,indices:Array.from(m.indices),indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
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
fn wrapped_single_character_v_string_repetition_state_context_and_consumers() {
    check(
        r#"let a=/([\q{a|b}])+/dv.exec('zabba!'),b=/(?:(?:[\q{a|b}]))+/v.exec('zabba!');a[0]==='abba'&&a[1]==='a'&&a.indices[1][0]===4&&a.indices[1][1]===5&&b.length===1&&b[0]==='abba'"#,
    );
    check(
        r#"let r=/(?<whole>(?<unit>([\q{a|b}]))+)/dvg,m=r.exec('zabba!');m.length===4&&m.groups.whole==='abba'&&m.groups.unit==='a'&&m[3]==='a'&&m.indices[1][0]===1&&m.indices[1][1]===5&&m.indices.groups.unit===m.indices[2]&&m.indices[3][0]===4&&r.lastIndex===5"#,
    );
    check(
        r#"let r=/(?<whole>(?<unit>([\q{a}--a])){0})/dvg,m=r.exec('a');m[0]===''&&m[1]===''&&m[2]===undefined&&m[3]===undefined&&Object.hasOwn(m,'2')&&Object.hasOwn(m,'3')&&Object.hasOwn(m.indices,'2')&&m.indices[2]===undefined&&m.indices[3]===undefined&&m.groups.whole===''&&m.groups.unit===undefined&&Object.hasOwn(m.groups,'unit')&&m.indices.groups.unit===undefined&&Object.hasOwn(m.indices.groups,'unit')&&m.indices[1][0]===0&&m.indices[1][1]===0&&r.lastIndex===0"#,
    );
    check(
        r#"let a=/([\q{a|b}])+/vg,b=/([\q{a|b}])+?/vg,m=a.exec('ab-ab!'),n=a.exec('ab-ab!');m[0]==='ab'&&m[1]==='b'&&m.index===0&&n.index===3&&a.lastIndex===5&&a.exec('ab-ab!')===null&&a.lastIndex===0&&b.exec('ab!')[0]==='a'&&b.lastIndex===1"#,
    );
    check(
        r#"let r=/\B([\q{a}]){1,2}?\b/dvy;r.lastIndex=0;let a=r.exec('aaa!');r.lastIndex=1;let b=r.exec('aaa!');a===null&&b.index===1&&b[0]==='aa'&&b[1]==='a'&&b.indices[1][0]===2&&r.lastIndex===3"#,
    );
    check(
        r#"/([\q{a}]){0001,0003}/v.exec('aaaa!')[0]==='aaa'&&/([\q{a}]){1,999999999999999999999999}/v.exec('aaa!')[1]==='a'&&!/([\q{a}]){999999999999999999999999}/v.test('a')"#,
    );
    check(
        r#"let a=/([\q{K}&&k])+/iv.exec('!KKk!'),b=/([^\q{K|s}])+?/iv.exec('Kſ!?');a[0]==='KKk'&&a[1]==='k'&&b.index===2&&b[0]==='!'&&b[1]==='!'"#,
    );
    check(
        r#"let r=/\b([\q{\uD83D\uDE00|x}--x]){1,2}\B/dvy;r.lastIndex=2;let m=r.exec('A😀😀!');m.index===1&&m[0]==='😀😀'&&m[1]==='😀'&&m.indices[1][0]===3&&m.indices[1][1]===5&&r.lastIndex===5"#,
    );
    check(
        r#"let m=/([\q{\uD800|x}--x]){2}/dv.exec('A\uD800\uD800B');m.index===1&&m[0].length===2&&m[1].length===1&&m[1].charCodeAt(0)===0xD800&&m.indices[1][0]===2&&m.indices[1][1]===3"#,
    );
    check(
        r#"let r=/^\B([\q{\r|\n}]){2}\B$/dvm,m=r.exec('x\n\r\n\rz');m.index===2&&m[0]==='\r\n'&&m[1]==='\n'&&m.indices[1][0]===3&&m.indices[1][1]===4&&!/^\B([\q{\r|\n}]){2}\B$/v.test('x\n\r\n\rz')"#,
    );
    check(
        r#"let trace='',r=/([\q{a|b}])+/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return 'zabb!'}});trace==='si'&&m[0]==='abb'&&m[1]==='b'&&m.index===1&&r.lastIndex===4"#,
    );
    check(
        r#"let seen=0,r=/([\q{a|b}])+/v;r.lastIndex={valueOf(){seen++;return 3}};r.exec('zabb!')[1]==='b'&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/([\q{K}&&k])+/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.source==='([\\q{K}&&k])+'&&c.flags==='dvy'&&c.lastIndex===0&&c.exec('K')===null&&r.exec('K')[1]==='K'&&r.lastIndex===7"#,
    );
    check(
        r#"let a=/([\q{a}])+/vy,b=/([\q{a}])+/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('zaa')}catch(e){x=e instanceof TypeError}try{b.exec('zaa')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>(?<unit>([\q{a|b}]))+)/dvg,a=[...'a-bb!'.matchAll(r)];a.length===2&&a[0].index===0&&a[1].index===2&&a[1].groups.whole==='bb'&&a[1].groups.unit==='b'&&a[1].indices[3][0]===3&&'a-bb!'.replace(r,'<$<whole>/$<unit>>')==='<a/a>-<bb/b>!'&&'za-bbz'.split(/([\q{a|b}])+/v).join('|')==='z|a|-|b|z'"#,
    );
    check(
        r#"let r=/([\q{a|b}])+/vg;r.lastIndex=4;'zab!'.search(r)===1&&r.lastIndex===4&&'ab-ab'.match(r).join('|')==='ab|ab'&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<unit>([\q{a}--a])){0}/dvg,a=[...'A😀B'.matchAll(r)];a.map(m=>m.index).join(',')==='0,1,3,4'&&a.every(m=>m[1]===undefined&&m[2]===undefined&&m.groups.unit===undefined&&Object.hasOwn(m.groups,'unit')&&Object.hasOwn(m.indices,'2'))&&r.lastIndex===0"#,
    );
    for pattern in [
        r"([\q{a}]){3,2}",
        r"([^\q{ab}]){0}",
        r"([\q{\d}])+",
        r"([\q{a}-b])+",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({code:?})}}catch(e){{caught=e instanceof SyntaxError}}caught&&marker===0"
        ));
    }
    for source in [
        r"/([\q{a}]|)+/v.test('a')",
        r"/([\q{a}&&[]&&\q{bc}]){0}/v.test('')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(true)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    for source in [
        r"/([\q{a}]b)+/v.test('ab')",
        r"/(a[\q{b}])+/v.test('ab')",
        r"/([\q{a}]|b)+/v.test('ab')",
        r"/(\b[\q{a}])+/v.test('a')",
        r"/([\q{a}]\b)+/v.test('a')",
        r"/(^[\q{a}])+/v.test('a')",
        r"/([\q{a}]$)+/v.test('a')",
        r"/(([\q{a}])+)+/v.test('a')",
        r"/([\q{a}])+b/v.test('ab')",
        r"/a([\q{b}])+/v.test('ab')",
        r"/([\q{}]){0}/v.test('')",
        r"/([\q{a|bc}]){0}/v.test('')",
        r"/([\q{a}--a--\p{RGI_Emoji}]){0}/v.test('')",
        r"/\b^([\q{a}])+/v.test('a')",
        r"/([\q{a}])+$$/v.test('a')",
        r"/\b\b([\q{a}])+/v.test('a')",
        r"/([\q{a}])+\b\b/v.test('a')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)), "{source}");
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>(?<unit>([\q{a|b}]))+)/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>(?<unit>([\\q{a|b}]))+)'&&kept.lastIndex===1&&kept.exec('zabb!').groups.unit==='b'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r#"var r=/(?<whole>(?<unit>([\q{z}]))+)/vg,marker=0"#)
        .unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn wrapped_single_character_v_string_repetition_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete JSON, participation and UTF16 spans were frozen before this source.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>(?<unit>([\q{a|b}]))+)"#,
            JsString::from_code_units(vec![122, 97, 98, 98, 97, 33]),
            "dvg",
            0usize,
            r#"{"matches":["abba","abba","a","a"],"index":1,"input":"zabba!","groups":{"whole":"abba","unit":"a"},"indices":[[1,5],[1,5],[4,5],[4,5]],"indexGroups":{"whole":[1,5],"unit":[4,5]},"lastIndex":5,"source":"(?<whole>(?<unit>([\\q{a|b}]))+)"}"#,
        ),
        (
            r#"(?<whole>\B(?<unit>([\q{a}])){1,2}?\b)"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvg",
            0usize,
            r#"{"matches":["aa","aa","a","a"],"index":1,"input":"aaa!","groups":{"whole":"aa","unit":"a"},"indices":[[1,3],[1,3],[2,3],[2,3]],"indexGroups":{"whole":[1,3],"unit":[2,3]},"lastIndex":3,"source":"(?<whole>\\B(?<unit>([\\q{a}])){1,2}?\\b)"}"#,
        ),
        (
            r#"(?<whole>(?<unit>([\q{a}--a])){0})"#,
            JsString::from_code_units(vec![97]),
            "dvg",
            0usize,
            r#"{"matches":["","",null,null],"index":0,"input":"a","groups":{"whole":""},"indices":[[0,0],[0,0],null,null],"indexGroups":{"whole":[0,0]},"lastIndex":0,"source":"(?<whole>(?<unit>([\\q{a}--a])){0})"}"#,
        ),
        (
            r#"(?<whole>(?<unit>([\q{a}])){999999999999999999999999})"#,
            JsString::from_code_units(vec![97]),
            "dvg",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>(?<unit>([\q{K}&&k]))+)"#,
            JsString::from_code_units(vec![33, 8490, 75, 107, 33]),
            "divg",
            0usize,
            r#"{"matches":["KKk","KKk","k","k"],"index":1,"input":"!KKk!","groups":{"whole":"KKk","unit":"k"},"indices":[[1,4],[1,4],[3,4],[3,4]],"indexGroups":{"whole":[1,4],"unit":[3,4]},"lastIndex":4,"source":"(?<whole>(?<unit>([\\q{K}&&k]))+)"}"#,
        ),
        (
            r#"(?<whole>(?<unit>([^\q{K|s}]))+?)"#,
            JsString::from_code_units(vec![8490, 383, 33, 63]),
            "divg",
            0usize,
            r#"{"matches":["!","!","!","!"],"index":2,"input":"Kſ!?","groups":{"whole":"!","unit":"!"},"indices":[[2,3],[2,3],[2,3],[2,3]],"indexGroups":{"whole":[2,3],"unit":[2,3]},"lastIndex":3,"source":"(?<whole>(?<unit>([^\\q{K|s}]))+?)"}"#,
        ),
        (
            r#"(?<whole>\b(?<unit>([\q{\uD83D\uDE00|x}--x])){1,2}\B)"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56832, 33]),
            "dvy",
            2usize,
            r#"{"matches":["😀😀","😀😀","😀","😀"],"index":1,"input":"A😀😀!","groups":{"whole":"😀😀","unit":"😀"},"indices":[[1,5],[1,5],[3,5],[3,5]],"indexGroups":{"whole":[1,5],"unit":[3,5]},"lastIndex":5,"source":"(?<whole>\\b(?<unit>([\\q{\\uD83D\\uDE00|x}--x])){1,2}\\B)"}"#,
        ),
        (
            r#"(?<whole>(?<unit>([\q{\uD800|x}--x])){2})"#,
            JsString::from_code_units(vec![65, 55296, 55296, 66]),
            "dvg",
            0usize,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800","\ud800"],"index":1,"input":"A\ud800\ud800B","groups":{"whole":"\ud800\ud800","unit":"\ud800"},"indices":[[1,3],[1,3],[2,3],[2,3]],"indexGroups":{"whole":[1,3],"unit":[2,3]},"lastIndex":3,"source":"(?<whole>(?<unit>([\\q{\\uD800|x}--x])){2})"}"#,
        ),
        (
            r#"^(?<whole>\B(?<unit>([\q{\r|\n}])){2}\B)$"#,
            JsString::from_code_units(vec![120, 10, 13, 10, 13, 122]),
            "dgmv",
            0usize,
            r#"{"matches":["\r\n","\r\n","\n","\n"],"index":2,"input":"x\n\r\n\rz","groups":{"whole":"\r\n","unit":"\n"},"indices":[[2,4],[2,4],[3,4],[3,4]],"indexGroups":{"whole":[2,4],"unit":[3,4]},"lastIndex":4,"source":"^(?<whole>\\B(?<unit>([\\q{\\r|\\n}])){2}\\B)$"}"#,
        ),
        (
            r#"(?<whole>(?<unit>([\q{b}])){2})"#,
            JsString::from_code_units(vec![97, 97, 98, 98]),
            "dvy",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
    ] {
        let script = format!(
            "let r=new RegExp({pattern:?},{flags:?});r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:Array.from(m),index:m.index,input:m.input,groups:m.groups,indices:Array.from(m.indices),indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
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
fn raw_empty_single_character_v_string_repetition_state_context_and_consumers() {
    check(
        r#"let a=/([\q{a}]|)*/dv.exec('aaa!'),b=/([\q{a}]|)*?/dv.exec('aaa!'),c=/([\q{a}]|)?/dv.exec('aaa!'),d=/([\q{a}]|)??/dv.exec('aaa!');a[0]==='aaa'&&a[1]==='a'&&b[0]===''&&b[1]===undefined&&c[0]==='a'&&d[0]===''&&d[1]===undefined"#,
    );
    check(
        r#"let a=/(?<whole>(?<unit>([\q{a}--a]|))+)/dv.exec('z'),b=/(?<whole>(?<unit>([\q{a}]|))*?)/dv.exec('a');a[0]===''&&a.groups.whole===''&&a[2]===''&&a[3]===''&&a.groups.unit===''&&a.indices[2][0]===0&&a.indices[2][1]===0&&b.groups.whole===''&&b[2]===undefined&&b[3]===undefined&&Object.hasOwn(b,'2')&&Object.hasOwn(b.indices,'3')&&Object.hasOwn(b.groups,'unit')&&b.groups.unit===undefined&&Object.hasOwn(b.indices.groups,'unit')&&b.indices.groups.unit===undefined"#,
    );
    check(
        r#"let a=/([\q{a}]|)+?\b/dv.exec('aa!'),b=/(|[\q{a}])+?/dv.exec('a'),c=/([\q{a}]|)+?/dv.exec('baaa');a.index===0&&a[0]==='aa'&&a[1]==='a'&&a.indices[1][0]===1&&b[0]===''&&b[1]===''&&c.index===0&&c[0]===''&&c[1]===''"#,
    );
    check(r#"let m=/([\q{\||\)}]|)+/v.exec('|)');m[0]==='|)'&&m[1]===')'"#);
    check(
        r#"let r=/([\q{a}]|)+/vg,a=r.exec('aa!'),b=r.exec('aa!');a[0]==='aa'&&r.lastIndex===2&&b.index===2&&b[0]===''&&b[1]===''&&r.exec('aa!').index===2&&(r.lastIndex=4,r.exec('aa!')===null)&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/\B([\q{a}]|)+?\b/dvy;r.lastIndex=0;let a=r.exec('aaa!');r.lastIndex=1;let b=r.exec('aaa!');a===null&&b.index===1&&b[0]==='aa'&&b[1]==='a'&&b.indices[1][0]===2&&r.lastIndex===3"#,
    );
    check(
        r#"let a=/^\b(|[\q{K}&&k])+?\b$/iv.exec('KK'),b=/([^\q{K|s}]|)+?/iv.exec('K!');a[0]==='KK'&&a[1]==='K'&&b.index===0&&b[0]===''&&b[1]===''"#,
    );
    check(
        r#"let r=/\b([\q{\uD83D\uDE00}]|)+\B/dvy;r.lastIndex=2;let m=r.exec('A😀😀!');m.index===1&&m[0]==='😀😀'&&m[1]==='😀'&&m.indices[1][0]===3&&m.indices[1][1]===5&&r.lastIndex===5"#,
    );
    check(
        r#"let r=/^\B(|[\q{\uD800}])+\B$/dgmvy;r.lastIndex=3;let m=r.exec('x\r\n\uD800\uD800\r\nz');m.index===3&&m[0].length===2&&m[1].length===1&&m[1].charCodeAt(0)===0xD800&&m.indices[1][0]===4&&m.indices[1][1]===5&&r.lastIndex===5"#,
    );
    check(
        r#"let trace='',r=/([\q{a}]|)+?/vgy;r.lastIndex={valueOf(){trace+='i';return 0}};let m=r.exec({toString(){trace+='s';return 'aaa'}});trace==='si'&&m[0]==='a'&&m[1]==='a'&&r.lastIndex===1"#,
    );
    check(
        r#"let seen=0,r=/([\q{a}]|)+/v;r.lastIndex={valueOf(){seen++;return 3}};r.exec('aaa')[1]==='a'&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/([\q{K}&&k]|)+/iv;r.lastIndex=7;let c=new RegExp(r,'dvy'),m=c.exec('K');c.source==='([\\q{K}&&k]|)+'&&c.flags==='dvy'&&m[0]===''&&m[1]===''&&c.lastIndex===0&&r.exec('K')[1]==='K'&&r.lastIndex===7"#,
    );
    check(
        r#"let a=/([\q{a}]|)+/vy,b=/([\q{a}]|)+/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('zaa')}catch(e){x=e instanceof TypeError}try{b.exec('z')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>(?<unit>([\q{a}]|))+)/dvg,a=[...'aa'.matchAll(r)];a.length===2&&a[0].groups.whole==='aa'&&a[0].groups.unit==='a'&&a[1].index===2&&a[1].groups.unit===''&&'aa'.replace(r,'<$<unit>>')==='<a><>'&&'aa'.split(/([\q{a}]|)+/v).join('|')==='|a|'&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/([\q{a}]|)+/vg;r.lastIndex=3;'ba'.search(r)===0&&r.lastIndex===3&&'aa'.match(r).join('|')==='aa|'&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<unit>(|[\q{a}]))+?/dvg,a=[...'A😀B'.matchAll(r)];a.map(m=>m.index).join(',')==='0,1,3,4'&&a.every(m=>m[1]===''&&m[2]===''&&m.groups.unit===''&&Object.hasOwn(m.indices,'2'))&&r.lastIndex===0"#,
    );
    for pattern in [
        r"([\q{\d}]|)+",
        r"([^\q{ab}]|)*",
        r"([\q{a}-b]|)+",
        r"([\q{a}]|)**",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('a');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({code:?})}}catch(e){{caught=e instanceof SyntaxError}}caught&&marker===0"
        ));
    }
    for source in [
        r"/([\q{a}]|){1}/v.test('')",
        r"/([\q{a}]|){1,}/v.test('a')",
        r"/(([\q{a}])|)+/v.test('a')",
        r"/([\q{a}]|())+/v.test('a')",
        r"/(?:[\q{a}]|(?:))+/v.test('a')",
        r"/([\q{a}]||)+/v.test('a')",
        r"/(|[\q{a}]|)+/v.test('a')",
        r"/([\q{a}]|b)+/v.test('ab')",
        r"/(\b[\q{a}]|)+/v.test('a')",
        r"/([\q{a}]|\b)+/v.test('a')",
        r"/([\q{a}]|)+b/v.test('ab')",
        r"/a([\q{b}]|)+/v.test('ab')",
        r"/([\q{}]|)+/v.test('')",
        r"/([\q{a|bc}]|)*/v.test('')",
        r"/([\q{a}&&[]&&\p{RGI_Emoji}]|)??/v.test('')",
        r"/\b^([\q{a}]|)+/v.test('a')",
        r"/([\q{a}]|)+$$/v.test('a')",
        r"/\b\b([\q{a}]|)+/v.test('a')",
        r"/([\q{a}]|)+\b\b/v.test('a')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)), "{source}");
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>(?<unit>([\q{a|b}]|))+)/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>(?<unit>([\\q{a|b}]|))+)'&&kept.lastIndex===1&&kept.exec('zabb!').groups.unit==='b'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    // This opt-in allowance targets construction; search/capture work has a direct unit.
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    assert!(matches!(
        realm
            .eval(r#"try{r=/(?<whole>(?<unit>([\q{a}]|))+?$)/vg}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("marker===0&&r===7"), Ok(Value::Boolean(true)));
}

#[test]
fn raw_empty_single_character_v_string_repetition_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete JSON, participation and UTF16 spans were frozen before this source.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>(?<unit>([\q{a}--a]|))+)"#,
            JsString::from_code_units(vec![122]),
            "dvg",
            0usize,
            r#"{"matches":["","","",""],"index":0,"input":"z","groups":{"whole":"","unit":""},"indices":[[0,0],[0,0],[0,0],[0,0]],"indexGroups":{"whole":[0,0],"unit":[0,0]},"lastIndex":0,"source":"(?<whole>(?<unit>([\\q{a}--a]|))+)"}"#,
        ),
        (
            r#"(?<whole>(?<unit>([\q{a}]|))*?)"#,
            JsString::from_code_units(vec![97]),
            "dvg",
            0usize,
            r#"{"matches":["","",null,null],"index":0,"input":"a","groups":{"whole":""},"indices":[[0,0],[0,0],null,null],"indexGroups":{"whole":[0,0]},"lastIndex":0,"source":"(?<whole>(?<unit>([\\q{a}]|))*?)"}"#,
        ),
        (
            r#"(?<whole>(?<unit>([\q{a}]|))+?\b)"#,
            JsString::from_code_units(vec![97, 97, 33]),
            "dvg",
            0usize,
            r#"{"matches":["aa","aa","a","a"],"index":0,"input":"aa!","groups":{"whole":"aa","unit":"a"},"indices":[[0,2],[0,2],[1,2],[1,2]],"indexGroups":{"whole":[0,2],"unit":[1,2]},"lastIndex":2,"source":"(?<whole>(?<unit>([\\q{a}]|))+?\\b)"}"#,
        ),
        (
            r#"(?<whole>(?<unit>(|[\q{a}]))+?)"#,
            JsString::from_code_units(vec![97]),
            "dvg",
            0usize,
            r#"{"matches":["","","",""],"index":0,"input":"a","groups":{"whole":"","unit":""},"indices":[[0,0],[0,0],[0,0],[0,0]],"indexGroups":{"whole":[0,0],"unit":[0,0]},"lastIndex":0,"source":"(?<whole>(?<unit>(|[\\q{a}]))+?)"}"#,
        ),
        (
            r#"(?<whole>(?<unit>([^\q{K|s}]|))+?)"#,
            JsString::from_code_units(vec![8490, 33]),
            "divg",
            0usize,
            r#"{"matches":["","","",""],"index":0,"input":"K!","groups":{"whole":"","unit":""},"indices":[[0,0],[0,0],[0,0],[0,0]],"indexGroups":{"whole":[0,0],"unit":[0,0]},"lastIndex":0,"source":"(?<whole>(?<unit>([^\\q{K|s}]|))+?)"}"#,
        ),
        (
            r#"^(?<whole>\b(?<unit>(|[\q{K}&&k]))+?\b)$"#,
            JsString::from_code_units(vec![8490, 75]),
            "divg",
            0usize,
            r#"{"matches":["KK","KK","K","K"],"index":0,"input":"KK","groups":{"whole":"KK","unit":"K"},"indices":[[0,2],[0,2],[1,2],[1,2]],"indexGroups":{"whole":[0,2],"unit":[1,2]},"lastIndex":2,"source":"^(?<whole>\\b(?<unit>(|[\\q{K}&&k]))+?\\b)$"}"#,
        ),
        (
            r#"(?<whole>\b(?<unit>([\q{\uD83D\uDE00}]|))+\B)"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56832, 33]),
            "dvy",
            2usize,
            r#"{"matches":["😀😀","😀😀","😀","😀"],"index":1,"input":"A😀😀!","groups":{"whole":"😀😀","unit":"😀"},"indices":[[1,5],[1,5],[3,5],[3,5]],"indexGroups":{"whole":[1,5],"unit":[3,5]},"lastIndex":5,"source":"(?<whole>\\b(?<unit>([\\q{\\uD83D\\uDE00}]|))+\\B)"}"#,
        ),
        (
            r#"^(?<whole>\B(?<unit>(|[\q{\uD800}]))+\B)$"#,
            JsString::from_code_units(vec![120, 13, 10, 55296, 55296, 13, 10, 122]),
            "dgmvy",
            3usize,
            r#"{"matches":["\ud800\ud800","\ud800\ud800","\ud800","\ud800"],"index":3,"input":"x\r\n\ud800\ud800\r\nz","groups":{"whole":"\ud800\ud800","unit":"\ud800"},"indices":[[3,5],[3,5],[4,5],[4,5]],"indexGroups":{"whole":[3,5],"unit":[4,5]},"lastIndex":5,"source":"^(?<whole>\\B(?<unit>(|[\\q{\\uD800}]))+\\B)$"}"#,
        ),
        (
            r#"(?<whole>\b(?<unit>([\q{a}--a]|))+\b)"#,
            JsString::from_code_units(vec![]),
            "dvg",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>\B(?<unit>([\q{a}]|))+?\b)"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvy",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
    ] {
        let script = format!(
            "let r=new RegExp({pattern:?},{flags:?});r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:Array.from(m),index:m.index,input:m.input,groups:m.groups,indices:Array.from(m.indices),indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
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
fn single_character_v_string_class_literal_state_context_and_consumers() {
    check(r#"/[\q{a}]b/v.exec('aab!').index===1&&!/[\q{a}]b/v.test('ax')&&!/[\q{a}]b/v.test('a')"#);
    check(
        r#"/[a\q{b|c}]d/v.test('cd')&&/[[\q{b|c}]d-f]a/v.test('ea')&&/[\q{a|b}&&[b-z]]a/v.test('ba')&&!/[\q{a|b}--b]a/v.test('ba')&&!/[a--a--\q{a}]c/v.test('ac')"#,
    );
    check(
        r#"let m=/[\q{a}]a\b/v.exec('aaa!');m.index===1&&m[0]==='aa'&&/^[\q{a}]b/v.exec('abx')[0]==='ab'"#,
    );
    check(
        r#"let r=/[\q{a}]a\b/dvgy;r.lastIndex=0;let a=r.exec('aaa!');r.lastIndex=1;let b=r.exec('aaa!');a===null&&b.index===1&&b[0]==='aa'&&b.indices[0][1]===3&&r.lastIndex===3&&r.exec('aaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[\q{a}]b/vg,a=r.exec('abab!'),b=r.exec('abab!');a.index===0&&b.index===2&&r.lastIndex===4&&r.exec('abab!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/[\q{K}&&k]s/iv.exec('!Kſ!')[0]==='Kſ'&&!/[\q{K}&&k]s/v.test('Kſ')&&!/[^\q{K|s}]s/iv.test('Kſ')"#,
    );
    check(
        r#"let m=/\B[^\q{K|s}]\$\B/iv.exec('KſA$');m.index===2&&m[0]==='A$'&&/^\b[\q{\!}\p{Any}]\$\B$/iv.test('K$')&&!/^\b[\q{\!}\p{Any}]\$\B$/v.test('K$')"#,
    );
    check(
        r#"let r=/\b[\q{\uD83D\uDE00|x}--x]\u{1f680}\B/dvy;r.lastIndex=2;let m=r.exec('A😀🚀!');m.index===1&&m[0]==='😀🚀'&&m.indices[0][0]===1&&m.indices[0][1]===5&&r.lastIndex===5"#,
    );
    check(
        r#"let m=/[\q{\uD800}]\\/v.exec('A\uD800\\B');m.index===1&&m[0].length===2&&m[0].charCodeAt(0)===0xD800&&m[0].charCodeAt(1)===92&&!/[\q{\uD800}]\\/v.test('😀\\')"#,
    );
    check(
        r#"let m=/^[\q{\]|\\|\[}]\$/mv.exec('zxy\r\n]$tail\u2028');m.index===5&&m[0]===']$'&&/^[\q{\]|\\|\[}]\$/v.test('\\$')&&/^[\q{\]|\\|\[}]\$/v.test('[$')"#,
    );
    check(
        r#"let m=/[\q{a|b}]c$/mv.exec('x\r\naabc\u2028z');m.index===5&&m[0]==='bc'&&!/[\q{a|b}]c$/v.test('x\r\naabc\u2028z')"#,
    );
    check(
        r#"let trace='',r=/[\q{a}]b/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return 'aab!'}});trace==='si'&&m[0]==='ab'&&m.index===1&&r.lastIndex===3"#,
    );
    check(
        r#"let seen=0,r=/[\q{a}]b/v;r.lastIndex={valueOf(){seen++;return 3}};r.exec('aab!').index===1&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[\q{K}&&k]s/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.source==='[\\q{K}&&k]s'&&c.flags==='dvy'&&c.lastIndex===0&&c.exec('Kſ')===null&&r.exec('Kſ')[0]==='Kſ'&&r.lastIndex===7"#,
    );
    check(
        r#"let a=/[\q{a}]b/vy,b=/[\q{a}]b/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aab!')}catch(e){x=e instanceof TypeError}try{b.exec('aab!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>([\q{a}]b))/dvg,a=[...'aab-ab!'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===4&&a[1].groups.whole==='ab'&&a[1].indices[2][1]===6&&'aab-ab!'.replace(r,'<$<whole>>')==='a<ab>-<ab>!'&&'aab-ab!'.split(/[\q{a}]b/v).join('|')==='a|-|!'"#,
    );
    check(
        r#"let r=/[\q{a}]b/vg;r.lastIndex=4;'aab!'.search(r)===1&&r.lastIndex===4&&'ab-ab'.match(r).join('|')==='ab|ab'&&r.lastIndex===0"#,
    );
    for pattern in [
        r"[\q{a}-b]c",
        r"[^\q{ab}]c",
        r"[\q{\d}]b",
        r"[\q{\_}]b",
        r"[\q{a}]\u{110000}",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({code:?})}}catch(e){{caught=e instanceof SyntaxError}}caught&&marker===0"
        ));
    }
    {
        let source = r"/[\q{a}][b]/v.test('ab')";
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(true)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    for source in [
        r"/([\q{a}])b/v.test('ab')",
        r"/[\q{a}](b)/v.test('ab')",
        r"/[\q{a}]+b/v.test('ab')",
        r"/[\q{a}]b+/v.test('ab')",
        r"/([\q{a}]b)+/v.test('ab')",
        r"/([\q{a}]|)+b/v.test('ab')",
        r"/a[\q{b}]c/v.test('abc')",
        r"/[\q{}]b/v.test('b')",
        r"/[\q{a|bc}]b/v.test('ab')",
        r"/[a--a--\q{a|bc}]b/v.test('ab')",
        r"/[\q{a}&&[]&&\p{RGI_Emoji}]b/v.test('ab')",
        r"/[\q{a}]b$$/v.test('ab')",
        r"/\b\b[\q{a}]b/v.test('ab')",
        r"/[\q{a}]b\b\b/v.test('ab')",
        r"/\b^[\q{a}]b/v.test('ab')",
        r"/[\q{a}]\bb/v.test('ab')",
        r"/[\q{a}]b$\b/v.test('ab')",
        r"/[\q{a}]b|c/v.test('ab')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)), "{source}");
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>([\q{a|b}&&b]a))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([\\q{a|b}&&b]a))'&&kept.lastIndex===1&&kept.exec('zba!').groups.whole==='ba'&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    // Explicit construction cancellation: no search setup is assumed reachable.
    // At over8000 units the new adapter's4N alone exceeds this opt-in budget.
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    let choices = "a|".repeat(4000) + "a";
    let source = format!(r#"try{{r=/[\q{{{choices}}}]b/vg}}catch{{marker=1}}finally{{marker=2}}"#);
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("marker===0&&r===7"), Ok(Value::Boolean(true)));
}

#[test]
fn single_character_v_string_class_literal_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // All ten complete JSON/UTF16 rows were handwritten and reviewed first.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([\q{a}]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvg",
            0usize,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([\\q{a}]a\\b))"}"#,
        ),
        (
            r#"(?<whole>([\q{a}]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvgy",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([\q{a}]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvgy",
            1usize,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([\\q{a}]a\\b))"}"#,
        ),
        (
            r#"(?<whole>([\q{a|b}]c$))"#,
            JsString::from_code_units(vec![120, 13, 10, 97, 97, 98, 99, 8232, 122]),
            "dgmv",
            0usize,
            r#"{"matches":["bc","bc","bc"],"index":5,"input":"x\r\naabc z","groups":{"whole":"bc"},"indices":[[5,7],[5,7],[5,7]],"indexGroups":{"whole":[5,7]},"lastIndex":7,"source":"(?<whole>([\\q{a|b}]c$))"}"#,
        ),
        (
            r#"(?<whole>(^[\q{\]|\\|\[}]\$))"#,
            JsString::from_code_units(vec![122, 120, 121, 13, 10, 93, 36, 116, 97, 105, 108, 8232]),
            "dgmv",
            0usize,
            r#"{"matches":["]$","]$","]$"],"index":5,"input":"zxy\r\n]$tail ","groups":{"whole":"]$"},"indices":[[5,7],[5,7],[5,7]],"indexGroups":{"whole":[5,7]},"lastIndex":7,"source":"(?<whole>(^[\\q{\\]|\\\\|\\[}]\\$))"}"#,
        ),
        (
            r#"(?<whole>([\q{K}&&k]\u{00000061}))"#,
            JsString::from_code_units(vec![8490, 65, 33]),
            "dgiv",
            0usize,
            r#"{"matches":["KA","KA","KA"],"index":0,"input":"KA!","groups":{"whole":"KA"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>([\\q{K}&&k]\\u{00000061}))"}"#,
        ),
        (
            r#"(?<whole>(\B[^\q{K|s}]\$\B))"#,
            JsString::from_code_units(vec![8490, 383, 65, 36]),
            "dgiv",
            0usize,
            r#"{"matches":["A$","A$","A$"],"index":2,"input":"KſA$","groups":{"whole":"A$"},"indices":[[2,4],[2,4],[2,4]],"indexGroups":{"whole":[2,4]},"lastIndex":4,"source":"(?<whole>(\\B[^\\q{K|s}]\\$\\B))"}"#,
        ),
        (
            r#"(?<whole>([\q{\uD83D\uDE00}]\uD83D\uDE80))"#,
            JsString::from_code_units(vec![55357, 56832, 55357, 56960, 33]),
            "dvgy",
            1usize,
            r#"{"matches":["😀🚀","😀🚀","😀🚀"],"index":0,"input":"😀🚀!","groups":{"whole":"😀🚀"},"indices":[[0,4],[0,4],[0,4]],"indexGroups":{"whole":[0,4]},"lastIndex":4,"source":"(?<whole>([\\q{\\uD83D\\uDE00}]\\uD83D\\uDE80))"}"#,
        ),
        (
            r#"(?<whole>([\q{\uD800}]\\))"#,
            JsString::from_code_units(vec![55296, 92]),
            "dvg",
            0usize,
            r#"{"matches":["\ud800\\","\ud800\\","\ud800\\"],"index":0,"input":"\ud800\\","groups":{"whole":"\ud800\\"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>([\\q{\\uD800}]\\\\))"}"#,
        ),
        (
            r#"(?<whole>([a--a--\q{a}]c))"#,
            JsString::from_code_units(vec![97, 99]),
            "dvg",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
    ] {
        let script = format!(
            "let r=new RegExp({pattern:?},{flags:?});r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:Array.from(m),index:m.index,input:m.input,groups:m.groups,indices:Array.from(m.indices),indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
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
fn single_character_v_string_then_flat_class_state_context_and_consumers() {
    check(
        r#"/[\q{a}][b]/v.exec('aab!').index===1&&!/[\q{a}][b]/v.test('ax')&&!/[\q{a}][b]/v.test('a')"#,
    );
    check(
        r#"/[a\q{b|c}][d]/v.test('cd')&&/[[\q{b|c}]d-f][a]/v.test('ea')&&/[\q{a|b}&&[b-z]][a]/v.test('ba')&&!/[\q{a|b}--b][a]/v.test('ba')&&!/[a--a--\q{a}][c]/v.test('ac')"#,
    );
    check(
        r#"let m=/[\q{a}][a]\b/v.exec('aaa!');m.index===1&&m[0]==='aa'&&/^[\q{a}][b]/v.exec('abx')[0]==='ab'"#,
    );
    check(
        r#"let r=/[\q{a}][a]\b/dvgy;r.lastIndex=0;let a=r.exec('aaa!');r.lastIndex=1;let b=r.exec('aaa!');a===null&&b.index===1&&b[0]==='aa'&&b.indices[0][1]===3&&r.lastIndex===3&&r.exec('aaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[\q{a}][b]/vg,a=r.exec('abab!'),b=r.exec('abab!');a.index===0&&b.index===2&&r.lastIndex===4&&r.exec('abab!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/[\q{K}&&k][s]/iv.exec('!Kſ!')[0]==='Kſ'&&!/[\q{K}&&k][s]/v.test('Kſ')&&!/[^\q{K|s}][s]/iv.test('Kſ')"#,
    );
    check(
        r#"let m=/\B[^\q{K|s}][\p{ASCII}]\B/iv.exec('KſA$');m.index===2&&m[0]==='A$'&&/^\b[\q{\!}\p{Any}][\$]\B$/iv.test('K$')&&!/^\b[\q{\!}\p{Any}][\$]\B$/v.test('K$')"#,
    );
    check(
        r#"let r=/\b[\q{\uD83D\uDE00|x}--x][\u{1f680}]\B/dvy;r.lastIndex=2;let m=r.exec('A😀🚀!');m.index===1&&m[0]==='😀🚀'&&m.indices[0][0]===1&&m.indices[0][1]===5&&r.lastIndex===5"#,
    );
    check(
        r#"let m=/[\q{\uD800}][\\]/v.exec('A\uD800\\B');m.index===1&&m[0].length===2&&m[0].charCodeAt(0)===0xD800&&m[0].charCodeAt(1)===92&&!/[\q{\uD800}][\\]/v.test('😀\\')"#,
    );
    check(
        r#"let m=/^[\q{\]|\\|\[}][\$]/mv.exec('zxy\r\n]$tail\u2028');m.index===5&&m[0]===']$'&&/^[\q{\]|\\|\[}][\$]/v.test('\\$')&&/^[\q{\]|\\|\[}][\$]/v.test('[$')"#,
    );
    check(
        r#"let m=/[\q{a|b}][c]$/mv.exec('x\r\naabc\u2028z');m.index===5&&m[0]==='bc'&&!/[\q{a|b}][c]$/v.test('x\r\naabc\u2028z')"#,
    );
    check(
        r#"let trace='',r=/[\q{a}][b]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return 'aab!'}});trace==='si'&&m[0]==='ab'&&m.index===1&&r.lastIndex===3"#,
    );
    check(
        r#"let seen=0,r=/[\q{a}][b]/v;r.lastIndex={valueOf(){seen++;return 3}};r.exec('aab!').index===1&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[\q{K}&&k][s]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.source==='[\\q{K}&&k][s]'&&c.flags==='dvy'&&c.lastIndex===0&&c.exec('Kſ')===null&&r.exec('Kſ')[0]==='Kſ'&&r.lastIndex===7"#,
    );
    check(
        r#"let a=/[\q{a}][b]/vy,b=/[\q{a}][b]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aab!')}catch(e){x=e instanceof TypeError}try{b.exec('aab!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>([\q{a}][b]))/dvg,a=[...'aab-ab!'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===4&&a[1].groups.whole==='ab'&&a[1].indices[2][1]===6&&'aab-ab!'.replace(r,'<$<whole>>')==='a<ab>-<ab>!'&&'aab-ab!'.split(/[\q{a}][b]/v).join('|')==='a|-|!'"#,
    );
    check(
        r#"let r=/[\q{a}][b]/vg;r.lastIndex=4;'aab!'.search(r)===1&&r.lastIndex===4&&'ab-ab'.match(r).join('|')==='ab|ab'&&r.lastIndex===0"#,
    );
    check(
        r#"/[\q{a|b}][b-d]/iv.test('aD')&&!/[\q{a|b}][b-d]/v.test('aD')&&/[\q{a}][\d]/v.test('a7')&&/[\q{a}][\D]/v.test('a!')"#,
    );
    check(
        r#"/[\q{a}][\p{ASCII}]/v.test('aA')&&!/[\q{a}][\p{ASCII}]/v.test('aK')&&/[\q{a}][\P{ASCII}]/v.test('aK')&&!/[\q{a}][]/v.test('ab')"#,
    );
    for pattern in [
        r"[\q{a}-b][c]",
        r"[^\q{ab}][c]",
        r"[\q{\d}][b]",
        r"[\q{\_}][b]",
        r"[\q{a}][\u{110000}]",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({code:?})}}catch(e){{caught=e instanceof SyntaxError}}caught&&marker===0"
        ));
    }
    {
        let source = r"/[a][\q{b}]/v.test('ab')";
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(true)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    {
        let source = r"/[\q{a}][\q{b}]/v.test('ab')";
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(true)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    for source in [
        r"/([\q{a}])[b]/v.test('ab')",
        r"/[\q{a}]([b])/v.test('ab')",
        r"/[\q{a}]+[b]/v.test('ab')",
        r"/[\q{a}][b]+/v.test('ab')",
        r"/(?:[\q{a}][b])+/v.test('ab')",
        r"/[\q{a}][b][c]/v.test('abc')",
        r"/[\q{}][b]/v.test('ab')",
        r"/[\q{a|bc}][b]/v.test('ab')",
        r"/[a--a--\q{a|bc}][b]/v.test('ab')",
        r"/[\q{a}&&[]&&\p{RGI_Emoji}][b]/v.test('ab')",
        r"/[a--a--\q{a}][\p{RGI_Emoji}]/v.test('ab')",
        r"/[\q{a}][b]$$/v.test('ab')",
        r"/\b\b[\q{a}][b]/v.test('ab')",
        r"/[\q{a}][b]\b\b/v.test('ab')",
        r"/\b^[\q{a}][b]/v.test('ab')",
        r"/[\q{a}]\b[b]/v.test('ab')",
        r"/[\q{a}][b]$\b/v.test('ab')",
        r"/[\q{a}][b]|c/v.test('ab')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)), "{source}");
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>([\q{a|b}&&b][a]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([\\q{a|b}&&b][a]))'&&kept.lastIndex===1&&kept.exec('zba!').groups.whole==='ba'&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    // Explicit construction cancellation: no search setup is assumed reachable.
    // At over8000 units the new adapter's4N alone exceeds this opt-in budget.
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    let choices = "a|".repeat(4000) + "a";
    let source =
        format!(r#"try{{r=/[\q{{{choices}}}][b]/vg}}catch{{marker=1}}finally{{marker=2}}"#);
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("marker===0&&r===7"), Ok(Value::Boolean(true)));
}

#[test]
fn single_character_v_string_then_flat_class_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // All ten complete JSON/UTF16 rows were handwritten and reviewed first.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([\q{a}][a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvg",
            0usize,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([\\q{a}][a]\\b))"}"#,
        ),
        (
            r#"(?<whole>([\q{a}][a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvgy",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([\q{a}][a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvgy",
            1usize,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([\\q{a}][a]\\b))"}"#,
        ),
        (
            r#"(?<whole>([\q{a|b}][c]$))"#,
            JsString::from_code_units(vec![120, 13, 10, 97, 97, 98, 99, 8232, 122]),
            "dgmv",
            0usize,
            r#"{"matches":["bc","bc","bc"],"index":5,"input":"x\r\naabc z","groups":{"whole":"bc"},"indices":[[5,7],[5,7],[5,7]],"indexGroups":{"whole":[5,7]},"lastIndex":7,"source":"(?<whole>([\\q{a|b}][c]$))"}"#,
        ),
        (
            r#"(?<whole>(^[\q{\]|\\|\[}][\$]))"#,
            JsString::from_code_units(vec![122, 120, 121, 13, 10, 93, 36, 116, 97, 105, 108, 8232]),
            "dgmv",
            0usize,
            r#"{"matches":["]$","]$","]$"],"index":5,"input":"zxy\r\n]$tail ","groups":{"whole":"]$"},"indices":[[5,7],[5,7],[5,7]],"indexGroups":{"whole":[5,7]},"lastIndex":7,"source":"(?<whole>(^[\\q{\\]|\\\\|\\[}][\\$]))"}"#,
        ),
        (
            r#"(?<whole>([\q{K}&&k][a]))"#,
            JsString::from_code_units(vec![8490, 65, 33]),
            "dgiv",
            0usize,
            r#"{"matches":["KA","KA","KA"],"index":0,"input":"KA!","groups":{"whole":"KA"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>([\\q{K}&&k][a]))"}"#,
        ),
        (
            r#"(?<whole>(\B[^\q{K|s}][\p{ASCII}]\B))"#,
            JsString::from_code_units(vec![8490, 383, 65, 36]),
            "dgiv",
            0usize,
            r#"{"matches":["A$","A$","A$"],"index":2,"input":"KſA$","groups":{"whole":"A$"},"indices":[[2,4],[2,4],[2,4]],"indexGroups":{"whole":[2,4]},"lastIndex":4,"source":"(?<whole>(\\B[^\\q{K|s}][\\p{ASCII}]\\B))"}"#,
        ),
        (
            r#"(?<whole>([\q{\uD83D\uDE00}][\uD83D\uDE80]))"#,
            JsString::from_code_units(vec![55357, 56832, 55357, 56960, 33]),
            "dvgy",
            1usize,
            r#"{"matches":["😀🚀","😀🚀","😀🚀"],"index":0,"input":"😀🚀!","groups":{"whole":"😀🚀"},"indices":[[0,4],[0,4],[0,4]],"indexGroups":{"whole":[0,4]},"lastIndex":4,"source":"(?<whole>([\\q{\\uD83D\\uDE00}][\\uD83D\\uDE80]))"}"#,
        ),
        (
            r#"(?<whole>([\q{\uD800}][\\]))"#,
            JsString::from_code_units(vec![55296, 92]),
            "dvg",
            0usize,
            r#"{"matches":["\ud800\\","\ud800\\","\ud800\\"],"index":0,"input":"\ud800\\","groups":{"whole":"\ud800\\"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>([\\q{\\uD800}][\\\\]))"}"#,
        ),
        (
            r#"(?<whole>([a--a--\q{a}][\p{ASCII}]))"#,
            JsString::from_code_units(vec![97, 99]),
            "dvg",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
    ] {
        let script = format!(
            "let r=new RegExp({pattern:?},{flags:?});r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:Array.from(m),index:m.index,input:m.input,groups:m.groups,indices:Array.from(m.indices),indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
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
fn flat_class_then_single_character_v_string_state_context_and_consumers() {
    check(
        r#"/[a][\q{b}]/v.exec('aab!').index===1&&!/[a][\q{b}]/v.test('ax')&&!/[a][\q{b}]/v.test('a')"#,
    );
    check(
        r#"/[d][a\q{b|c}]/v.test('dc')&&/[a][[\q{b|c}]d-f]/v.test('ae')&&/[a][\q{a|b}&&[b-z]]/v.test('ab')&&!/[a][\q{a|b}--b]/v.test('ab')&&!/[][a--a--\q{a}]/v.test('aa')"#,
    );
    check(
        r#"let m=/[a][\q{a}]\b/v.exec('aaa!');m.index===1&&m[0]==='aa'&&/^[a][\q{b}]/v.exec('abx')[0]==='ab'"#,
    );
    check(
        r#"let r=/[a][\q{a}]\b/dvgy;r.lastIndex=0;let a=r.exec('aaa!');r.lastIndex=1;let b=r.exec('aaa!');a===null&&b.index===1&&b[0]==='aa'&&b.indices[0][1]===3&&r.lastIndex===3&&r.exec('aaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[a][\q{b}]/vg,a=r.exec('abab!'),b=r.exec('abab!');a.index===0&&b.index===2&&r.lastIndex===4&&r.exec('abab!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/[s][\q{K}&&k]/iv.exec('!ſK!')[0]==='ſK'&&!/[s][\q{K}&&k]/v.test('ſK')&&!/[s][^\q{K|s}]/iv.test('ſK')"#,
    );
    check(
        r#"let m=/\B[\p{ASCII}][^\q{K|s}]\B/iv.exec('KſA$');m.index===2&&m[0]==='A$'&&/^\b[\p{ASCII}][\q{\!}\p{Any}]\B$/iv.test('K$')&&!/^\b[\p{ASCII}][\q{\!}\p{Any}]\B$/v.test('K$')"#,
    );
    check(
        r#"let r=/\b[\u{1f600}][\q{\uD83D\uDE80|x}--x]\B/dvy;r.lastIndex=2;let m=r.exec('A😀🚀!');m.index===1&&m[0]==='😀🚀'&&m.indices[0][0]===1&&m.indices[0][1]===5&&r.lastIndex===5"#,
    );
    check(
        r#"let m=/[\\][\q{\uD800}]/v.exec('A\\\uD800B');m.index===1&&m[0].length===2&&m[0].charCodeAt(0)===92&&m[0].charCodeAt(1)===0xD800&&!/[\\][\q{\uD800}]/v.test('\\😀')"#,
    );
    check(
        r#"let m=/^[\$][\q{\]|\\|\[}]/mv.exec('zxy\r\n$]tail\u2028');m.index===5&&m[0]==='$]'&&/^[\$][\q{\]|\\|\[}]/v.test('$\\')&&/^[\$][\q{\]|\\|\[}]/v.test('$[')"#,
    );
    check(
        r#"let m=/[a-c][\q{c|d}]$/mv.exec('x\r\naabc\u2028z');m.index===5&&m[0]==='bc'&&!/[a-c][\q{c|d}]$/v.test('x\r\naabc\u2028z')"#,
    );
    check(
        r#"let trace='',r=/[a][\q{b}]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return 'aab!'}});trace==='si'&&m[0]==='ab'&&m.index===1&&r.lastIndex===3"#,
    );
    check(
        r#"let seen=0,r=/[a][\q{b}]/v;r.lastIndex={valueOf(){seen++;return 3}};r.exec('aab!').index===1&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[s][\q{K}&&k]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.source==='[s][\\q{K}&&k]'&&c.flags==='dvy'&&c.lastIndex===0&&c.exec('ſK')===null&&r.exec('ſK')[0]==='ſK'&&r.lastIndex===7"#,
    );
    check(
        r#"let a=/[a][\q{b}]/vy,b=/[a][\q{b}]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aab!')}catch(e){x=e instanceof TypeError}try{b.exec('aab!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>([a][\q{b}]))/dvg,a=[...'aab-ab!'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===4&&a[1].groups.whole==='ab'&&a[1].indices[2][1]===6&&'aab-ab!'.replace(r,'<$<whole>>')==='a<ab>-<ab>!'&&'aab-ab!'.split(/[a][\q{b}]/v).join('|')==='a|-|!'"#,
    );
    check(
        r#"let r=/[a][\q{b}]/vg;r.lastIndex=4;'aab!'.search(r)===1&&r.lastIndex===4&&'ab-ab'.match(r).join('|')==='ab|ab'&&r.lastIndex===0"#,
    );
    check(
        r#"/[b-d][\q{a|b}]/iv.test('Da')&&!/[b-d][\q{a|b}]/v.test('Da')&&/[\d][\q{a}]/v.test('7a')&&/[\D][\q{a}]/v.test('!a')"#,
    );
    check(
        r#"/[\p{ASCII}][\q{a}]/v.test('Aa')&&!/[\p{ASCII}][\q{a}]/v.test('Ka')&&/[\P{ASCII}][\q{a}]/v.test('Ka')&&!/[][\q{a}]/v.test('ab')"#,
    );
    check(
        r#"/[\p{ASCII}][\q{a}]/iv.test('KA')&&/[\p{ASCII}][\q{a}]/iv.test('ſA')&&!/[\p{ASCII}][\q{a}]/v.test('ſA')"#,
    );
    for pattern in [
        r"[c][\q{a}-b]",
        r"[c][^\q{ab}]",
        r"[b][\q{\d}]",
        r"[b][\q{\_}]",
        r"[\u{110000}][\q{a}]",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({code:?})}}catch(e){{caught=e instanceof SyntaxError}}caught&&marker===0"
        ));
    }
    {
        let source = r"/[\q{a}][\q{b}]/v.test('ab')";
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(true)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    for source in [
        r"/([a])[\q{b}]/v.test('ab')",
        r"/[a]([\q{b}])/v.test('ab')",
        r"/[a]+[\q{b}]/v.test('ab')",
        r"/[a][\q{b}]+/v.test('ab')",
        r"/(?:[a][\q{b}])+/v.test('ab')",
        r"/[a][\q{b}][c]/v.test('abc')",
        r"/[][\q{}]/v.test('ab')",
        r"/[][\q{a|bc}]/v.test('ab')",
        r"/[][a--a--\q{a|bc}]/v.test('ab')",
        r"/[][\q{a}&&[]&&\p{RGI_Emoji}]/v.test('ab')",
        r"/[\p{RGI_Emoji}][\q{a}]/v.test('ab')",
        r"/[a][\q{b}]$$/v.test('ab')",
        r"/\b\b[a][\q{b}]/v.test('ab')",
        r"/[a][\q{b}]\b\b/v.test('ab')",
        r"/\b^[a][\q{b}]/v.test('ab')",
        r"/[a]\b[\q{b}]/v.test('ab')",
        r"/[a][\q{b}]$\b/v.test('ab')",
        r"/[a][\q{b}]|c/v.test('ab')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)), "{source}");
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>([a][\q{a|b}&&b]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([a][\\q{a|b}&&b]))'&&kept.lastIndex===1&&kept.exec('aab!').groups.whole==='ab'&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    // Construction cancellation only: no search setup is assumed reachable.
    // Over8000 source units make the new adapter's4N exceed this opt-in budget;
    // an earlier validated-construction charge may fail first.
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    let choices = "a|".repeat(4000) + "a";
    let source =
        format!(r#"try{{r=/[a][\q{{{choices}}}]/vg}}catch{{marker=1}}finally{{marker=2}}"#);
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("marker===0&&r===7"), Ok(Value::Boolean(true)));

    // Exact original Boolean controls now complete through checked-prefix/q-suffix proof.
    for (source, expected) in [
        (r"/[[a]][\q{b}]/v.test('ab')", true),
        (r"/[a&&a][\q{b}]/v.test('ab')", true),
        (r"/[a--a][\q{b}]/v.test('ab')", false),
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(expected)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    check(
        r#"let r=/(?<whole>([[a]][\q{b}]))/dvg,m=r.exec("ab");Array.isArray(m)&&Object.getPrototypeOf(m)===Array.prototype&&m.length===3&&m[0]==="ab"&&m[1]==="ab"&&m[2]==="ab"&&m.index===0&&m.input==="ab"&&Object.getPrototypeOf(m.groups)===null&&m.groups.whole==="ab"&&m.indices.length===3&&m.indices[0][0]===0&&m.indices[0][1]===2&&m.indices[1][0]===0&&m.indices[1][1]===2&&m.indices[2][0]===0&&m.indices[2][1]===2&&m.indices.groups.whole[0]===0&&m.indices.groups.whole[1]===2&&r.source==="(?<whole>([[a]][\\q{b}]))"&&r.flags==='dgv'&&r.lastIndex===2&&r.exec("ab")===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>([a&&a][\q{b}]))/dvg,m=r.exec("ab");Array.isArray(m)&&Object.getPrototypeOf(m)===Array.prototype&&m.length===3&&m[0]==="ab"&&m[1]==="ab"&&m[2]==="ab"&&m.index===0&&m.input==="ab"&&Object.getPrototypeOf(m.groups)===null&&m.groups.whole==="ab"&&m.indices.length===3&&m.indices[0][0]===0&&m.indices[0][1]===2&&m.indices[1][0]===0&&m.indices[1][1]===2&&m.indices[2][0]===0&&m.indices[2][1]===2&&m.indices.groups.whole[0]===0&&m.indices.groups.whole[1]===2&&r.source==="(?<whole>([a&&a][\\q{b}]))"&&r.flags==='dgv'&&r.lastIndex===2&&r.exec("ab")===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>([a--a][\q{b}]))/dvg;r.lastIndex=1;let m=r.exec("ab");m===null&&r.lastIndex===0&&r.source==="(?<whole>([a--a][\\q{b}]))"&&r.flags==='dgv'"#,
    );
}

#[test]
fn flat_class_then_single_character_v_string_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // All ten complete JSON/UTF16 rows were handwritten and reviewed first.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([a][\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvg",
            0usize,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([a][\\q{a}]\\b))"}"#,
        ),
        (
            r#"(?<whole>([a][\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvgy",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([a][\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvgy",
            1usize,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([a][\\q{a}]\\b))"}"#,
        ),
        (
            r#"(?<whole>([a-c][\q{c|d}]$))"#,
            JsString::from_code_units(vec![120, 13, 10, 97, 97, 98, 99, 8232, 122]),
            "dgmv",
            0usize,
            r#"{"matches":["bc","bc","bc"],"index":5,"input":"x\r\naabc z","groups":{"whole":"bc"},"indices":[[5,7],[5,7],[5,7]],"indexGroups":{"whole":[5,7]},"lastIndex":7,"source":"(?<whole>([a-c][\\q{c|d}]$))"}"#,
        ),
        (
            r#"(?<whole>(^[\$][\q{\]|\\|\[}]))"#,
            JsString::from_code_units(vec![122, 120, 121, 13, 10, 36, 93, 116, 97, 105, 108, 8232]),
            "dgmv",
            0usize,
            r#"{"matches":["$]","$]","$]"],"index":5,"input":"zxy\r\n$]tail ","groups":{"whole":"$]"},"indices":[[5,7],[5,7],[5,7]],"indexGroups":{"whole":[5,7]},"lastIndex":7,"source":"(?<whole>(^[\\$][\\q{\\]|\\\\|\\[}]))"}"#,
        ),
        (
            r#"(?<whole>([a][\q{K}&&k]))"#,
            JsString::from_code_units(vec![65, 8490, 33]),
            "dgiv",
            0usize,
            r#"{"matches":["AK","AK","AK"],"index":0,"input":"AK!","groups":{"whole":"AK"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>([a][\\q{K}&&k]))"}"#,
        ),
        (
            r#"(?<whole>(\B[\p{ASCII}][^\q{K|s}]\B))"#,
            JsString::from_code_units(vec![8490, 383, 65, 36]),
            "dgiv",
            0usize,
            r#"{"matches":["A$","A$","A$"],"index":2,"input":"KſA$","groups":{"whole":"A$"},"indices":[[2,4],[2,4],[2,4]],"indexGroups":{"whole":[2,4]},"lastIndex":4,"source":"(?<whole>(\\B[\\p{ASCII}][^\\q{K|s}]\\B))"}"#,
        ),
        (
            r#"(?<whole>([\uD83D\uDE00][\q{\uD83D\uDE80}]))"#,
            JsString::from_code_units(vec![55357, 56832, 55357, 56960, 33]),
            "dvgy",
            1usize,
            r#"{"matches":["😀🚀","😀🚀","😀🚀"],"index":0,"input":"😀🚀!","groups":{"whole":"😀🚀"},"indices":[[0,4],[0,4],[0,4]],"indexGroups":{"whole":[0,4]},"lastIndex":4,"source":"(?<whole>([\\uD83D\\uDE00][\\q{\\uD83D\\uDE80}]))"}"#,
        ),
        (
            r#"(?<whole>([\\][\q{\uD800}]))"#,
            JsString::from_code_units(vec![92, 55296]),
            "dvg",
            0usize,
            r#"{"matches":["\\\ud800","\\\ud800","\\\ud800"],"index":0,"input":"\\\ud800","groups":{"whole":"\\\ud800"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>([\\\\][\\q{\\uD800}]))"}"#,
        ),
        (
            r#"(?<whole>([][a--a--\q{a}]))"#,
            JsString::from_code_units(vec![97, 99]),
            "dvg",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
    ] {
        let script = format!(
            "let r=new RegExp({pattern:?},{flags:?});r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:Array.from(m),index:m.index,input:m.input,groups:m.groups,indices:Array.from(m.indices),indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
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
fn single_character_v_string_class_pair_state_context_and_consumers() {
    check(
        r#"/[\q{a}][\q{b}]/v.exec('aab!').index===1&&!/[\q{a}][\q{b}]/v.test('ax')&&!/[\q{a}][\q{b}]/v.test('a')"#,
    );
    check(
        r#"/[a\q{b|c}][\q{d|e}]/v.test('cd')&&/[[\q{b|c}]d-f][\q{a}]/v.test('ea')&&/[\q{a|b}&&[b-z]][\q{a}]/v.test('ba')&&!/[\q{a|b}--b][\q{a}]/v.test('ba')&&!/[a--a--\q{a}][\q{b}--b]/v.test('ab')"#,
    );
    check(
        r#"let m=/[\q{a}][\q{a}]\b/v.exec('aaa!');m.index===1&&m[0]==='aa'&&/^[\q{a}][\q{b}]/v.exec('abx')[0]==='ab'"#,
    );
    check(
        r#"let r=/[\q{a}][\q{a}]\b/dvgy;r.lastIndex=0;let a=r.exec('aaa!');r.lastIndex=1;let b=r.exec('aaa!');a===null&&b.index===1&&b[0]==='aa'&&b.indices[0][1]===3&&r.lastIndex===3&&r.exec('aaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[\q{a}][\q{b}]/vg,a=r.exec('abab!'),b=r.exec('abab!');a.index===0&&b.index===2&&r.lastIndex===4&&r.exec('abab!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/[\q{s}][\q{K}&&k]/iv.exec('!ſK!')[0]==='ſK'&&!/[\q{s}][\q{K}&&k]/v.test('ſK')&&!/[\q{s}][^\q{K|s}]/iv.test('ſK')"#,
    );
    check(
        r#"let m=/\B[\q{K|s|a}][^\q{K|s}]\B/iv.exec('KſA$');m.index===2&&m[0]==='A$'&&/^\b[\q{\!}\p{ASCII}][\q{\!}\p{Any}]\B$/iv.test('K$')&&!/^\b[\q{\!}\p{ASCII}][\q{\!}\p{Any}]\B$/v.test('K$')"#,
    );
    check(
        r#"let r=/\b[\q{\uD83D\uDE00}][\q{\u{1f680}|x}--x]\B/dvy;r.lastIndex=2;let m=r.exec('A😀🚀!');m.index===1&&m[0]==='😀🚀'&&m.indices[0][0]===1&&m.indices[0][1]===5&&r.lastIndex===5"#,
    );
    check(
        r#"let m=/[\q{\uD800}][\q{\\}]/v.exec('A\uD800\\B');m.index===1&&m[0].length===2&&m[0].charCodeAt(0)===0xD800&&m[0].charCodeAt(1)===92&&!/[\q{\uD800}][\q{\\}]/v.test('😀\\')"#,
    );
    check(
        r#"let m=/^[\q{\$}][\q{\]|\\|\[}]/mv.exec('zxy\r\n$]tail\u2028');m.index===5&&m[0]==='$]'&&/^[\q{\$}][\q{\]|\\|\[}]/v.test('$\\')&&/^[\q{\$}][\q{\]|\\|\[}]/v.test('$[')"#,
    );
    check(
        r#"let m=/[\q{a|b}][\q{c|d}]$/mv.exec('x\r\naabc\u2028z');m.index===5&&m[0]==='bc'&&!/[\q{a|b}][\q{c|d}]$/v.test('x\r\naabc\u2028z')"#,
    );
    check(
        r#"let trace='',r=/[\q{a}][\q{b}]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return 'aab!'}});trace==='si'&&m[0]==='ab'&&m.index===1&&r.lastIndex===3"#,
    );
    check(
        r#"let seen=0,r=/[\q{a}][\q{b}]/v;r.lastIndex={valueOf(){seen++;return 3}};r.exec('aab!').index===1&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[\q{s}][\q{K}&&k]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.source==='[\\q{s}][\\q{K}&&k]'&&c.flags==='dvy'&&c.lastIndex===0&&c.exec('ſK')===null&&r.exec('ſK')[0]==='ſK'&&r.lastIndex===7"#,
    );
    check(
        r#"let a=/[\q{a}][\q{b}]/vy,b=/[\q{a}][\q{b}]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aab!')}catch(e){x=e instanceof TypeError}try{b.exec('aab!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>([\q{a}][\q{b}]))/dvg,a=[...'aab-ab!'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===4&&a[1].groups.whole==='ab'&&a[1].indices[2][1]===6&&'aab-ab!'.replace(r,'<$<whole>>')==='a<ab>-<ab>!'&&'aab-ab!'.split(/[\q{a}][\q{b}]/v).join('|')==='a|-|!'"#,
    );
    check(
        r#"let r=/[\q{a}][\q{b}]/vg;r.lastIndex=4;'aab!'.search(r)===1&&r.lastIndex===4&&'ab-ab'.match(r).join('|')==='ab|ab'&&r.lastIndex===0"#,
    );
    check(
        r#"/[b-d\q{a}][\q{a|b}]/iv.test('Da')&&!/[b-d\q{a}][\q{a|b}]/v.test('Da')&&/[\d\q{a}][\q{b}]/v.test('7b')&&/[\D\q{a}][\q{b}]/v.test('!b')"#,
    );
    check(
        r#"/[\q{a}\p{ASCII}][\q{a}]/v.test('Aa')&&!/[\q{a}\p{ASCII}][\q{a}]/v.test('Ka')&&/[\q{\u{1f600}}\P{ASCII}][\q{a}]/v.test('Ka')&&/[\q{a}\p{ASCII}][\q{a}]/iv.test('KA')&&/[\q{a}\p{ASCII}][\q{a}]/iv.test('ſA')"#,
    );
    for pattern in [
        r"[\q{a}-b][\q{c}]",
        r"[\q{c}][^\q{ab}]",
        r"[\q{\d}][\q{b}]",
        r"[\q{a}][\q{\_}]",
        r"[\q{a}][\q{\u{110000}}]",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({code:?})}}catch(e){{caught=e instanceof SyntaxError}}caught&&marker===0"
        ));
    }
    for source in [
        r"/([\q{a}])[\q{b}]/v.test('ab')",
        r"/[\q{a}]([\q{b}])/v.test('ab')",
        r"/[\q{a}]+[\q{b}]/v.test('ab')",
        r"/[\q{a}][\q{b}]+/v.test('ab')",
        r"/(?:[\q{a}][\q{b}])+/v.test('ab')",
        r"/[\q{a}][\q{b}][c]/v.test('abc')",
        r"/[\q{}][\q{b}]/v.test('ab')",
        r"/[\q{a|bc}][\q{b}]/v.test('ab')",
        r"/[\q{a}\p{RGI_Emoji}][\q{b}]/v.test('ab')",
        r"/[a--a--\q{a}][\q{b|cd}]/v.test('ab')",
        r"/[a--a--\q{a}][\q{b}&&[]&&\p{RGI_Emoji}]/v.test('ab')",
        r"/[a--a--\q{a}][b--b--\q{b|cd}]/v.test('ab')",
        r"/[\q{a}&&[]][\q{b|}]/v.test('ab')",
        r"/[\q{a}][\q{b}]$$/v.test('ab')",
        r"/\b\b[\q{a}][\q{b}]/v.test('ab')",
        r"/[\q{a}][\q{b}]\b\b/v.test('ab')",
        r"/\b^[\q{a}][\q{b}]/v.test('ab')",
        r"/[\q{a}]\b[\q{b}]/v.test('ab')",
        r"/[\q{a}][\q{b}]$\b/v.test('ab')",
        r"/[\q{a}][\q{b}]|c/v.test('ab')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)), "{source}");
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>([\q{a|b}&&b][\q{a}]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([\\q{a|b}&&b][\\q{a}]))'&&kept.lastIndex===1&&kept.exec('zba!').groups.whole==='ba'&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    // Construction cancellation only: no search setup is assumed reachable.
    // Over8000 source units make this adapter's4N exceed the opt-in budget;
    // an earlier validated-construction charge may fail first.
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    let choices = "a|".repeat(4000) + "a";
    let source =
        format!(r#"try{{r=/[\q{{{choices}}}][\q{{b}}]/vg}}catch{{marker=1}}finally{{marker=2}}"#);
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("marker===0&&r===7"), Ok(Value::Boolean(true)));

    // Exact original Boolean controls now complete through checked-prefix/q-suffix proof.
    for (source, expected) in [
        (r"/[[a]][\q{b}]/v.test('ab')", true),
        (r"/[a&&a][\q{b}]/v.test('ab')", true),
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(expected)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    check(
        r#"let r=/(?<whole>([[a]][\q{b}]))/dvg,m=r.exec("ab");Array.isArray(m)&&Object.getPrototypeOf(m)===Array.prototype&&m.length===3&&m[0]==="ab"&&m[1]==="ab"&&m[2]==="ab"&&m.index===0&&m.input==="ab"&&Object.getPrototypeOf(m.groups)===null&&m.groups.whole==="ab"&&m.indices.length===3&&m.indices[0][0]===0&&m.indices[0][1]===2&&m.indices[1][0]===0&&m.indices[1][1]===2&&m.indices[2][0]===0&&m.indices[2][1]===2&&m.indices.groups.whole[0]===0&&m.indices.groups.whole[1]===2&&r.source==="(?<whole>([[a]][\\q{b}]))"&&r.flags==='dgv'&&r.lastIndex===2&&r.exec("ab")===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>([a&&a][\q{b}]))/dvg,m=r.exec("ab");Array.isArray(m)&&Object.getPrototypeOf(m)===Array.prototype&&m.length===3&&m[0]==="ab"&&m[1]==="ab"&&m[2]==="ab"&&m.index===0&&m.input==="ab"&&Object.getPrototypeOf(m.groups)===null&&m.groups.whole==="ab"&&m.indices.length===3&&m.indices[0][0]===0&&m.indices[0][1]===2&&m.indices[1][0]===0&&m.indices[1][1]===2&&m.indices[2][0]===0&&m.indices[2][1]===2&&m.indices.groups.whole[0]===0&&m.indices.groups.whole[1]===2&&r.source==="(?<whole>([a&&a][\\q{b}]))"&&r.flags==='dgv'&&r.lastIndex===2&&r.exec("ab")===null&&r.lastIndex===0"#,
    );
}

#[test]
fn single_character_v_string_class_pair_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // All ten complete JSON/UTF16 rows were handwritten and reviewed first.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([\q{a}][\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvg",
            0usize,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([\\q{a}][\\q{a}]\\b))"}"#,
        ),
        (
            r#"(?<whole>([\q{a}][\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvgy",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([\q{a}][\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvgy",
            1usize,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([\\q{a}][\\q{a}]\\b))"}"#,
        ),
        (
            r#"(?<whole>([\q{a|b}][\q{c|d}]$))"#,
            JsString::from_code_units(vec![120, 13, 10, 97, 97, 98, 99, 8232, 122]),
            "dgmv",
            0usize,
            r#"{"matches":["bc","bc","bc"],"index":5,"input":"x\r\naabc z","groups":{"whole":"bc"},"indices":[[5,7],[5,7],[5,7]],"indexGroups":{"whole":[5,7]},"lastIndex":7,"source":"(?<whole>([\\q{a|b}][\\q{c|d}]$))"}"#,
        ),
        (
            r#"(?<whole>(^[\q{\$}][\q{\]|\\|\[}]))"#,
            JsString::from_code_units(vec![122, 120, 121, 13, 10, 36, 93, 116, 97, 105, 108, 8232]),
            "dgmv",
            0usize,
            r#"{"matches":["$]","$]","$]"],"index":5,"input":"zxy\r\n$]tail ","groups":{"whole":"$]"},"indices":[[5,7],[5,7],[5,7]],"indexGroups":{"whole":[5,7]},"lastIndex":7,"source":"(?<whole>(^[\\q{\\$}][\\q{\\]|\\\\|\\[}]))"}"#,
        ),
        (
            r#"(?<whole>([\q{s}][\q{K}&&k]))"#,
            JsString::from_code_units(vec![383, 8490, 33]),
            "dgiv",
            0usize,
            r#"{"matches":["ſK","ſK","ſK"],"index":0,"input":"ſK!","groups":{"whole":"ſK"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>([\\q{s}][\\q{K}&&k]))"}"#,
        ),
        (
            r#"(?<whole>(\B[\q{K|s|a}][^\q{K|s}]\B))"#,
            JsString::from_code_units(vec![8490, 383, 65, 36]),
            "dgiv",
            0usize,
            r#"{"matches":["A$","A$","A$"],"index":2,"input":"KſA$","groups":{"whole":"A$"},"indices":[[2,4],[2,4],[2,4]],"indexGroups":{"whole":[2,4]},"lastIndex":4,"source":"(?<whole>(\\B[\\q{K|s|a}][^\\q{K|s}]\\B))"}"#,
        ),
        (
            r#"(?<whole>([\q{\uD83D\uDE00}][\q{\uD83D\uDE80}]))"#,
            JsString::from_code_units(vec![55357, 56832, 55357, 56960, 33]),
            "dvgy",
            1usize,
            r#"{"matches":["😀🚀","😀🚀","😀🚀"],"index":0,"input":"😀🚀!","groups":{"whole":"😀🚀"},"indices":[[0,4],[0,4],[0,4]],"indexGroups":{"whole":[0,4]},"lastIndex":4,"source":"(?<whole>([\\q{\\uD83D\\uDE00}][\\q{\\uD83D\\uDE80}]))"}"#,
        ),
        (
            r#"(?<whole>([\q{\uD800}][\q{\\}]))"#,
            JsString::from_code_units(vec![55296, 92]),
            "dvg",
            0usize,
            r#"{"matches":["\ud800\\","\ud800\\","\ud800\\"],"index":0,"input":"\ud800\\","groups":{"whole":"\ud800\\"},"indices":[[0,2],[0,2],[0,2]],"indexGroups":{"whole":[0,2]},"lastIndex":2,"source":"(?<whole>([\\q{\\uD800}][\\q{\\\\}]))"}"#,
        ),
        (
            r#"(?<whole>([a--a--\q{a}][\q{b}--b]))"#,
            JsString::from_code_units(vec![97, 98]),
            "dvg",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
    ] {
        let script = format!(
            "let r=new RegExp({pattern:?},{flags:?});r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:Array.from(m),index:m.index,input:m.input,groups:m.groups,indices:Array.from(m.indices),indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
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
fn checked_class_then_single_character_v_string_state_context_and_consumers() {
    check(
        r#"let m=/[[a]][\q{b}]/dv.exec('aab!');Array.isArray(m)&&m.index===1&&m[0]==='ab'&&m.indices[0][0]===1&&m.indices[0][1]===3&&!/[[a]][\q{b}]/v.test('ax')&&!/[[a]][\q{b}]/v.test('a')"#,
    );
    check(
        r#"/[a&&a][\q{b|c}--c]/v.test('ab')&&/[[a-c]--b][\q{a|b}]/v.test('ca')&&!/[a--a][\q{b}]/v.test('ab')&&!/[[a]][\q{b}--b]/v.test('ab')"#,
    );
    check(
        r#"let m=/[a&&a][\q{a}]\b/v.exec('aaa!'),p=/\b[[a]][\q{b}]/v.exec('abx');m.index===1&&m[0]==='aa'&&p.index===0&&p[0]==='ab'"#,
    );
    check(
        r#"let r=/[a&&a][\q{a}]\b/dvgy;r.lastIndex=0;let a=r.exec('aaa!');r.lastIndex=1;let b=r.exec('aaa!');a===null&&b.index===1&&b[0]==='aa'&&b.indices[0][1]===3&&r.lastIndex===3&&r.exec('aaa!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"let r=/[[a]][\q{b}]/vg,a=r.exec('abab!'),b=r.exec('abab!');a.index===0&&b.index===2&&r.lastIndex===4&&r.exec('abab!')===null&&r.lastIndex===0"#,
    );
    check(
        r#"/[s&&s][\q{K}&&k]/iv.exec('!ſK!')[0]==='ſK'&&!/[s&&s][\q{K}&&k]/v.test('ſK')&&!/[s&&s][^\q{K|s}]/iv.test('ſK')"#,
    );
    check(
        r#"let m=/\B[[\p{Any}]][^\q{K|s}]\B/iv.exec('KſA$');m.index===2&&m[0]==='A$'&&m.input==='KſA$'"#,
    );
    check(
        r#"/^\B[[\p{Any}]][\q{\!}\p{Any}]\b$/iv.test('!K')&&!/^\B[[\p{Any}]][\q{\!}\p{Any}]\b$/v.test('!K')&&/^\B[[\p{Any}]][\q{\!}\p{Any}]\b$/iv.test('!ſ')&&!/^\B[[\p{Any}]][\q{\!}\p{Any}]\b$/v.test('!ſ')"#,
    );
    check(
        r#"let r=/\b[[\u{1f600}]][\q{\uD83D\uDE80|x}--x]\B/dvy;r.lastIndex=2;let m=r.exec('A😀🚀!');m.index===1&&m[0]==='😀🚀'&&m.indices[0][0]===1&&m.indices[0][1]===5&&r.lastIndex===5"#,
    );
    check(
        r#"let m=/[[\\]][\q{\uD800}]/v.exec('A\\\uD800B');m.index===1&&m[0].length===2&&m[0].charCodeAt(0)===92&&m[0].charCodeAt(1)===0xD800&&!/[[\\]][\q{\uD800}]/v.test('\\😀')"#,
    );
    check(
        r#"let m=/^[[\$]][\q{\]|\\|\[}]/mv.exec('zxy\r\n$]tail\u2028');m.index===5&&m[0]==='$]'&&/^[[\$]][\q{\]|\\|\[}]/v.test('$\\')&&/^[[\$]][\q{\]|\\|\[}]/v.test('$[')"#,
    );
    check(
        r#"let m=/^[[a]][\q{b}]$/mv.exec('x\r\nab\u2028z');m.index===3&&m[0]==='ab'&&!/^[[a]][\q{b}]$/v.test('x\r\nab\u2028z')&&!/^[[a]][\q{b}]$/v.test('abx')"#,
    );
    check(
        r#"let trace='',r=/[[a]][\q{b}]/vy;r.lastIndex={valueOf(){trace+='i';return 1}};let m=r.exec({toString(){trace+='s';return 'aab!'}});trace==='si'&&m[0]==='ab'&&m.index===1&&r.lastIndex===3"#,
    );
    check(
        r#"let seen=0,r=/[[a]][\q{b}]/v;r.lastIndex={valueOf(){seen++;return 3}};r.exec('aab!').index===1&&seen===1&&typeof r.lastIndex==='object'"#,
    );
    check(
        r#"let r=/[s&&s][\q{K}&&k]/iv;r.lastIndex=7;let c=new RegExp(r,'dvy');c.source==='[s&&s][\\q{K}&&k]'&&c.flags==='dvy'&&c.lastIndex===0&&c.exec('ſK')===null&&r.exec('ſK')[0]==='ſK'&&r.lastIndex===7"#,
    );
    check(
        r#"let a=/[[a]][\q{b}]/vy,b=/[[a]][\q{b}]/vy;a.lastIndex=1;Object.defineProperty(a,'lastIndex',{writable:false});Object.defineProperty(b,'lastIndex',{writable:false});let x=false,y=false;try{a.exec('aab!')}catch(e){x=e instanceof TypeError}try{b.exec('aab!')}catch(e){y=e instanceof TypeError}x&&y&&a.lastIndex===1&&b.lastIndex===0"#,
    );
    check(
        r#"let r=/(?<whole>([a&&a][\q{b}]))/dvg,a=[...'aab-ab!'.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===4&&a[1].groups.whole==='ab'&&a[1].indices[2][1]===6&&'aab-ab!'.replace(r,'<$<whole>>')==='a<ab>-<ab>!'&&'aab-ab!'.split(/[[a]][\q{b}]/v).join('|')==='a|-|!'"#,
    );
    check(
        r#"let r=/[[a]][\q{b}]/vg;r.lastIndex=4;'aab!'.search(r)===1&&r.lastIndex===4&&'ab-ab'.match(r).join('|')==='ab|ab'&&r.lastIndex===0"#,
    );
    check(
        r#"let seen='',r=/(?<whole>([[a]][\q{b}]))/vg;let out='aab-ab!'.replace(r,(m,a,b,i,s,g)=>{seen+=i+':'+m+':'+a+':'+b+':'+g.whole+';';return '#'});out==='a#-#!'&&seen==='1:ab:ab:ab:ab;4:ab:ab:ab:ab;'&&r.lastIndex===0"#,
    );
    check(
        r#"/[^a&&a][\q{b}]/v.test('!b')&&!/[^a&&a][\q{b}]/v.test('ab')&&/[[a]][\q{b|c}&&[b-z]]/v.test('ab')&&!/[[a]][\q{b|c}&&[b-z]]/v.test('ad')"#,
    );
    check(
        r#"/[[a]][\q{\b|\x62|\u{00000062}}]/v.test('a\b')&&/[[a]][\q{\b|\x62|\u{00000062}}]/v.test('ab')&&!/[[a]][\q{\b|\x62|\u{00000062}}]/v.test('a')"#,
    );
    check(
        r#"/[a][\q{b}]/v.test('ab')&&/[\q{a}][\q{b}]/v.test('ab')&&/[[a]][[b]]/v.test('ab')&&/[[a]][b]/v.test('ab')"#,
    );
    check(
        r#"/[[\x61]][\q{\]|\[|\\}]/v.test('a]')&&/[[\x61]][\q{\]|\[|\\}]/v.test('a[')&&/[[\x61]][\q{\]|\[|\\}]/v.test('a\\')&&/[[\u{1f600}]][\q{🚀}]/v.test('😀🚀')"#,
    );
    for pattern in [
        r"[a&&a][^\q{ab}]",
        r"[a&&a][\q{\d}]",
        r"[a&&a][\q{\_}]",
        r"[a&&a][\q{\u{110000}}]",
        r"[a&&a][\q{a}-b]",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('ab');").as_str());
        check(&format!(
            "let marker=0,caught=false;try{{eval({code:?})}}catch(e){{caught=e instanceof SyntaxError}}caught&&marker===0"
        ));
    }
    for source in [
        r"/([[a]])[\q{b}]/v.test('ab')",
        r"/[[a]]([\q{b}])/v.test('ab')",
        r"/(?:[[a]][\q{b}])+/v.test('ab')",
        r"/[[a]]+[\q{b}]/v.test('ab')",
        r"/[[a]][\q{b}]+/v.test('ab')",
        r"/[[a]][\q{b}][c]/v.test('abc')",
        r"/[[a]][\q{b}]c/v.test('abc')",
        r"/a[[a]][\q{b}]/v.test('aab')",
        r"/[[a]]\b[\q{b}]/v.test('ab')",
        r"/\b^[[a]][\q{b}]/v.test('ab')",
        r"/[[a]][\q{b}]$\b/v.test('ab')",
        r"/\b\b[[a]][\q{b}]/v.test('ab')",
        r"/[[a]][\q{b}]\b\b/v.test('ab')",
        r"/[[a]][\q{b}]$$/v.test('ab')",
        r"/[[a]][\q{}]/v.test('ab')",
        r"/[[a]][\q{b|cd}]/v.test('ab')",
        r"/[a--a][\q{b}&&[]&&\p{RGI_Emoji}]/v.test('ab')",
        r"/[a--a][b--b--\q{b|cd}]/v.test('ab')",
        r"/[[\p{RGI_Emoji}]][\q{b}]/v.test('ab')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(1.0)), "{source}");
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>([a&&a][\q{b|c}--c]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([a&&a][\\q{b|c}--c]))'&&kept.lastIndex===1&&kept.exec('zab!').groups.whole==='ab'&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    // Construction cancellation only; no search setup is assumed reachable.
    // More than8000 source units force this entry's4N beyond the opt-in limit.
    // Earlier validated-construction charges may reach that same limit first.
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    let choices = "a|".repeat(4000) + "a";
    let source =
        format!(r#"try{{r=/[a&&a][\q{{{choices}}}]/vg}}catch{{marker=1}}finally{{marker=2}}"#);
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("marker===0&&r===7"), Ok(Value::Boolean(true)));
}

#[test]
fn checked_class_then_single_character_v_string_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // All ten full JSON/UTF16 rows were handwritten before source authoring.
    for (pattern, input, flags, start, expected) in [
        (
            r#"(?<whole>([[a]][\q{b}]))"#,
            JsString::from_code_units(vec![97, 97, 98, 33]),
            "dvg",
            0usize,
            r#"{"matches":["ab","ab","ab"],"index":1,"input":"aab!","groups":{"whole":"ab"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([[a]][\\q{b}]))"}"#,
        ),
        (
            r#"(?<whole>([a&&a][\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvg",
            0usize,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([a&&a][\\q{a}]\\b))"}"#,
        ),
        (
            r#"(?<whole>([a&&a][\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvy",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
        (
            r#"(?<whole>([a&&a][\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvy",
            1usize,
            r#"{"matches":["aa","aa","aa"],"index":1,"input":"aaa!","groups":{"whole":"aa"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([a&&a][\\q{a}]\\b))"}"#,
        ),
        (
            r#"(?<whole>([s&&s][\q{K}&&k]))"#,
            JsString::from_code_units(vec![33, 383, 8490, 33]),
            "divg",
            0usize,
            r#"{"matches":["ſK","ſK","ſK"],"index":1,"input":"!ſK!","groups":{"whole":"ſK"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([s&&s][\\q{K}&&k]))"}"#,
        ),
        (
            r#"(?<whole>(\B[[\p{Any}]][^\q{K|s}]\B))"#,
            JsString::from_code_units(vec![8490, 383, 65, 36]),
            "divg",
            0usize,
            r#"{"matches":["A$","A$","A$"],"index":2,"input":"KſA$","groups":{"whole":"A$"},"indices":[[2,4],[2,4],[2,4]],"indexGroups":{"whole":[2,4]},"lastIndex":4,"source":"(?<whole>(\\B[[\\p{Any}]][^\\q{K|s}]\\B))"}"#,
        ),
        (
            r#"(?<whole>(\b[[\u{1f600}]][\q{\uD83D\uDE80|x}--x]\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56960, 33]),
            "dvy",
            2usize,
            r#"{"matches":["😀🚀","😀🚀","😀🚀"],"index":1,"input":"A😀🚀!","groups":{"whole":"😀🚀"},"indices":[[1,5],[1,5],[1,5]],"indexGroups":{"whole":[1,5]},"lastIndex":5,"source":"(?<whole>(\\b[[\\u{1f600}]][\\q{\\uD83D\\uDE80|x}--x]\\B))"}"#,
        ),
        (
            r#"(?<whole>([[\\]][\q{\uD800}]))"#,
            JsString::from_code_units(vec![65, 92, 55296, 66]),
            "dvg",
            0usize,
            r#"{"matches":["\\\ud800","\\\ud800","\\\ud800"],"index":1,"input":"A\\\ud800B","groups":{"whole":"\\\ud800"},"indices":[[1,3],[1,3],[1,3]],"indexGroups":{"whole":[1,3]},"lastIndex":3,"source":"(?<whole>([[\\\\]][\\q{\\uD800}]))"}"#,
        ),
        (
            r#"(?<whole>(^[[\$]][\q{\]|\\|\[}]))"#,
            JsString::from_code_units(vec![122, 120, 121, 13, 10, 36, 93, 116, 97, 105, 108, 8232]),
            "dgmv",
            0usize,
            r#"{"matches":["$]","$]","$]"],"index":5,"input":"zxy\r\n$]tail ","groups":{"whole":"$]"},"indices":[[5,7],[5,7],[5,7]],"indexGroups":{"whole":[5,7]},"lastIndex":7,"source":"(?<whole>(^[[\\$]][\\q{\\]|\\\\|\\[}]))"}"#,
        ),
        (
            r#"(?<whole>([a--a][\q{b}]))"#,
            JsString::from_code_units(vec![97, 98, 33]),
            "dvg",
            0usize,
            r#"{"match":null,"lastIndex":0}"#,
        ),
    ] {
        let script = format!(
            "let r=new RegExp({pattern:?},{flags:?});r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:Array.from(m),index:m.index,input:m.input,groups:m.groups,indices:Array.from(m.indices),indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
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
