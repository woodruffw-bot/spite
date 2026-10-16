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
        "[a--b]c",
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
    for source in [
        r"/\b^[[K]]/v.test('K')",
        r"/[[K]]$\b/v.test('K')",
        r"/\b[a--b]+\b/v.test('a')",
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
