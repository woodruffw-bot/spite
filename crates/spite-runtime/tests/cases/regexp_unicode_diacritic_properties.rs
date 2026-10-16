//! Diacritic and its exact Dia alias over complete Unicode characters.
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
fn unicode_diacritic_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Diacritic}",
        r"\P{Dia}",
        r"[^\p{Dia}]",
        r"[\p{Dia}K]",
        r"[\P{Diacritic}K]",
        r"^(?<x>[\p{Dia}K])$",
        r"(?<x>\P{Diacritic})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{5d}\u{5e}\u{5f}\u{60}\u{61}\u{a7}\u{a8}\u{a9}\u{ae}\u{af}\u{b0}\u{b3}\u{b4}\u{b5}\u{b6}\u{b7}\u{b8}\u{b9}\u{2af}\u{2b0}\u{345}\u{34e}\u{34f}\u{350}\u{357}\u{358}\u{35c}\u{35d}\u{362}\u{363}\u{373}\u{374}\u{375}\u{376}\u{379}\u{37a}\u{37b}\u{383}\u{384}\u{385}\u{386}\u{399}\u{3b9}\u{482}\u{483}\u{487}\u{488}\u{557}\u{559}\u{55a}\u{58a}\u{58d}\u{590}\u{591}\u{5bd}\u{5be}\u{5bf}\u{5c0}\u{5c1}\u{5c2}\u{5c3}\u{5c4}\u{5c5}\u{5c6}\u{5c7}\u{5ca}\u{64a}\u{64b}\u{652}\u{653}\u{655}\u{658}\u{659}\u{6de}\u{6df}\u{6e0}\u{6e1}\u{6e3}\u{6e4}\u{6e5}\u{6e6}\u{6e7}\u{6e9}\u{6ea}\u{6ec}\u{6ed}\u{72f}\u{730}\u{74a}\u{74b}\u{7a5}\u{7a6}\u{7b0}\u{7b1}\u{7ea}\u{7eb}\u{7f5}\u{7f6}\u{817}\u{818}\u{819}\u{81a}\u{897}\u{898}\u{89f}\u{8a0}\u{8c8}\u{8c9}\u{8d4}\u{8e2}\u{8e3}\u{8fe}\u{8ff}\u{93b}\u{93c}\u{93d}\u{94c}\u{94d}\u{94e}\u{950}\u{951}\u{954}\u{955}\u{970}\u{971}\u{972}\u{9bb}\u{9bc}\u{9bd}\u{9cc}\u{9cd}\u{9ce}\u{a3b}\u{a3c}\u{a3d}\u{a4c}\u{a4d}\u{a4e}\u{abb}\u{abc}\u{abd}\u{acc}\u{acd}\u{ace}\u{afc}\u{afd}\u{aff}\u{b00}\u{b3b}\u{b3c}\u{b3d}\u{b4c}\u{b4d}\u{b4e}\u{b52}\u{b55}\u{b56}\u{bcc}\u{bcd}\u{bce}\u{c3b}\u{c3c}\u{c3d}\u{c4c}\u{c4d}\u{c4e}\u{cbb}\u{cbc}\u{cbd}\u{ccc}\u{ccd}\u{cce}\u{d3a}\u{d3b}\u{d3c}\u{d3d}\u{d4c}\u{d4d}\u{d4e}\u{dc9}\u{dca}\u{dcb}\u{e39}\u{e3a}\u{e3b}\u{e46}\u{e47}\u{e4c}\u{e4d}\u{e4e}\u{e4f}\u{eb9}\u{eba}\u{ebb}\u{ec7}\u{ec8}\u{ecc}\u{ecd}\u{f17}\u{f18}\u{f19}\u{f1a}\u{f34}\u{f35}\u{f36}\u{f37}\u{f38}\u{f39}\u{f3a}\u{f3d}\u{f3e}\u{f3f}\u{f40}\u{f81}\u{f82}\u{f84}\u{f85}\u{f86}\u{f87}\u{f88}\u{fc5}\u{fc6}\u{fc7}\u{1036}\u{1037}\u{1038}\u{1039}\u{103a}\u{103b}\u{1062}\u{1063}\u{1064}\u{1065}\u{1068}\u{1069}\u{106d}\u{106e}\u{1086}\u{1087}\u{108d}\u{108e}\u{108f}\u{1090}\u{1099}\u{109a}\u{109b}\u{109c}\u{135c}\u{135d}\u{135f}\u{1360}\u{1713}\u{1714}\u{1715}\u{1716}\u{1733}\u{1734}\u{1735}\u{17c8}\u{17c9}\u{17d3}\u{17d4}\u{17dc}\u{17dd}\u{17de}\u{1938}\u{1939}\u{193b}\u{193c}\u{1a5f}\u{1a60}\u{1a61}\u{1a74}\u{1a75}\u{1a7c}\u{1a7d}\u{1a7e}\u{1a7f}\u{1a80}\u{1aaf}\u{1ab0}\u{1abe}\u{1abf}\u{1ac0}\u{1ac1}\u{1acb}\u{1acc}\u{1ace}\u{1acf}\u{1af1}\u{1b33}\u{1b34}\u{1b35}\u{1b43}\u{1b44}\u{1b45}\u{1b6a}\u{1b6b}\u{1b73}\u{1b74}\u{1ba9}\u{1baa}\u{1bab}\u{1bac}\u{1be5}\u{1be6}\u{1be7}\u{1bf1}\u{1bf2}\u{1bf3}\u{1bf4}\u{1c35}\u{1c36}\u{1c37}\u{1c38}\u{1c77}\u{1c78}\u{1c7d}\u{1c7e}\u{1ccf}\u{1cd0}\u{1ce8}\u{1ce9}\u{1cec}\u{1ced}\u{1cee}\u{1cf3}\u{1cf4}\u{1cf5}\u{1cf6}\u{1cf7}\u{1cf9}\u{1cfa}\u{1d2b}\u{1d2c}\u{1d6a}\u{1d6b}\u{1d9a}\u{1d9b}\u{1dbe}\u{1dbf}\u{1dc3}\u{1dc4}\u{1dcf}\u{1dd0}\u{1df4}\u{1df5}\u{1dff}\u{1e00}\u{1fbc}\u{1fbd}\u{1fbe}\u{1fbf}\u{1fc1}\u{1fc2}\u{1fcc}\u{1fcd}\u{1fcf}\u{1fd0}\u{1fdc}\u{1fdd}\u{1fdf}\u{1fe0}\u{1fec}\u{1fed}\u{1fef}\u{1ff0}\u{1ffc}\u{1ffd}\u{1ffe}\u{1fff}\u{208e}\u{2090}\u{2cee}\u{2cef}\u{2cf1}\u{2cf2}\u{2e2e}\u{2e2f}\u{2e30}\u{3029}\u{302a}\u{302f}\u{3030}\u{3098}\u{3099}\u{309c}\u{309d}\u{30fb}\u{30fc}\u{30fd}\u{a66e}\u{a66f}\u{a670}\u{a67b}\u{a67c}\u{a67d}\u{a67e}\u{a67f}\u{a680}\u{a69b}\u{a69c}\u{a69d}\u{a69e}\u{a6ef}\u{a6f0}\u{a6f1}\u{a6f2}\u{a6ff}\u{a700}\u{a721}\u{a722}\u{a787}\u{a788}\u{a78a}\u{a78b}\u{a7f0}\u{a7f1}\u{a7f2}\u{a7f7}\u{a7f8}\u{a7f9}\u{a7fa}\u{a805}\u{a806}\u{a807}\u{a82b}\u{a82c}\u{a82d}\u{a8c3}\u{a8c4}\u{a8c5}\u{a8df}\u{a8e0}\u{a8f1}\u{a8f2}\u{a92a}\u{a92b}\u{a92e}\u{a92f}\u{a952}\u{a953}\u{a954}\u{a9b2}\u{a9b3}\u{a9b4}\u{a9bf}\u{a9c0}\u{a9c1}\u{a9e4}\u{a9e5}\u{a9e6}\u{aa7a}\u{aa7b}\u{aa7d}\u{aa7e}\u{aabe}\u{aabf}\u{aac2}\u{aac3}\u{aaf5}\u{aaf6}\u{aaf7}\u{ab5a}\u{ab5b}\u{ab5f}\u{ab60}\u{ab68}\u{ab69}\u{ab6b}\u{ab6c}\u{abeb}\u{abec}\u{abed}\u{abee}\u{fb1d}\u{fb1e}\u{fb1f}\u{fe1f}\u{fe20}\u{fe2f}\u{fe30}\u{ff3d}\u{ff3e}\u{ff3f}\u{ff40}\u{ff41}\u{ff6f}\u{ff70}\u{ff71}\u{ff9d}\u{ff9e}\u{ff9f}\u{ffa0}\u{ffe2}\u{ffe3}\u{ffe4}\u{102df}\u{102e0}\u{102e1}\u{1077f}\u{10780}\u{10785}\u{10786}\u{10787}\u{107b0}\u{107b1}\u{107b2}\u{107ba}\u{107bb}\u{10a37}\u{10a38}\u{10a3a}\u{10a3b}\u{10a3e}\u{10a3f}\u{10a40}\u{10ae4}\u{10ae5}\u{10ae6}\u{10ae7}\u{10d21}\u{10d22}\u{10d27}\u{10d28}\u{10d4d}\u{10d4e}\u{10d4f}\u{10d68}\u{10d69}\u{10d6d}\u{10d6e}\u{10eca}\u{10ed0}\u{10eef}\u{10ef8}\u{10efa}\u{10efb}\u{10efc}\u{10efd}\u{10eff}\u{10f00}\u{10f45}\u{10f46}\u{10f50}\u{10f51}\u{10f81}\u{10f82}\u{10f85}\u{10f86}\u{11045}\u{11046}\u{11047}\u{1106f}\u{11070}\u{11071}\u{110b8}\u{110b9}\u{110ba}\u{110bb}\u{11132}\u{11133}\u{11134}\u{11135}\u{11172}\u{11173}\u{11174}\u{111bf}\u{111c0}\u{111c1}\u{111c9}\u{111ca}\u{111cc}\u{111cd}\u{11234}\u{11235}\u{11236}\u{11237}\u{112e8}\u{112e9}\u{112ea}\u{112eb}\u{1133a}\u{1133b}\u{1133c}\u{1133d}\u{1134c}\u{1134d}\u{1134e}\u{11365}\u{11366}\u{1136c}\u{1136d}\u{1136f}\u{11370}\u{11374}\u{11375}\u{113cd}\u{113ce}\u{113d0}\u{113d1}\u{113d2}\u{113d3}\u{113d4}\u{113e0}\u{113e1}\u{113e2}\u{113e3}\u{11441}\u{11442}\u{11443}\u{11445}\u{11446}\u{11447}\u{114c1}\u{114c2}\u{114c3}\u{114c4}\u{115be}\u{115bf}\u{115c0}\u{115c1}\u{1163e}\u{1163f}\u{11640}\u{116b5}\u{116b6}\u{116b7}\u{116b8}\u{1172a}\u{1172b}\u{1172c}\u{11838}\u{11839}\u{1183a}\u{1183b}\u{1193c}\u{1193d}\u{1193e}\u{1193f}\u{11942}\u{11943}\u{11944}\u{119df}\u{119e0}\u{119e1}\u{11a33}\u{11a34}\u{11a35}\u{11a46}\u{11a47}\u{11a48}\u{11a98}\u{11a99}\u{11a9a}\u{11c3e}\u{11c3f}\u{11c40}\u{11d41}\u{11d42}\u{11d43}\u{11d44}\u{11d45}\u{11d46}\u{11d96}\u{11d97}\u{11d98}\u{11dd8}\u{11dd9}\u{11dda}\u{11f40}\u{11f41}\u{11f42}\u{11f43}\u{11f59}\u{11f5a}\u{11f5b}\u{13446}\u{13447}\u{13455}\u{13456}\u{1612e}\u{1612f}\u{16130}\u{16aef}\u{16af0}\u{16af4}\u{16af5}\u{16b2f}\u{16b30}\u{16b36}\u{16b37}\u{16d6a}\u{16d6b}\u{16d6c}\u{16d6d}\u{16f8e}\u{16f8f}\u{16f9f}\u{16fa0}\u{16fef}\u{16ff0}\u{16ff1}\u{16ff2}\u{1afef}\u{1aff0}\u{1aff3}\u{1aff4}\u{1aff5}\u{1affb}\u{1affc}\u{1affd}\u{1affe}\u{1afff}\u{1ceff}\u{1cf00}\u{1cf2d}\u{1cf2e}\u{1cf2f}\u{1cf30}\u{1cf46}\u{1cf47}\u{1d166}\u{1d167}\u{1d169}\u{1d16a}\u{1d16c}\u{1d16d}\u{1d172}\u{1d173}\u{1d17a}\u{1d17b}\u{1d182}\u{1d183}\u{1d184}\u{1d185}\u{1d18b}\u{1d18c}\u{1d1a9}\u{1d1aa}\u{1d1ad}\u{1d1ae}\u{1d24f}\u{1d253}\u{1d25a}\u{1d25d}\u{1dfcc}\u{1dfd0}\u{1e000}\u{1e02f}\u{1e030}\u{1e06d}\u{1e06e}\u{1e12f}\u{1e130}\u{1e136}\u{1e137}\u{1e2ad}\u{1e2ae}\u{1e2af}\u{1e2eb}\u{1e2ec}\u{1e2ef}\u{1e2f0}\u{1e5ed}\u{1e5ee}\u{1e5ef}\u{1e5f0}\u{1e8cf}\u{1e8d0}\u{1e8d6}\u{1e8d7}\u{1e943}\u{1e944}\u{1e946}\u{1e947}\u{1e948}\u{1e94a}\u{1e94b}\u{feff}ſK\n",
                "X\r\n\u{300}\u{2028}\u{301}\u{2029}",
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
fn unicode_diacritic_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let p=/^\p{{Dia}}$/i{mode},n=/^\P{{Dia}}$/i{mode};'\u0345\u0399\u03B9\u1FBE'.split('').every(c=>p.test(c)&&n.test(c)===('{mode}'==='u'))"
        ));
        check(&format!(
            r"let p=/[\p{{Dia}}K]/i{mode},n=/[\P{{Dia}}K]/i{mode},x=/[^\P{{Dia}}K]/i{mode};p.test('K')&&n.test('K')&&!x.test('K')&&'\u0345\u0399\u03B9\u1FBE'.split('').every(c=>p.test(c)&&n.test(c)===('{mode}'==='u')&&x.test(c)===('{mode}'==='v'))"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Diacritic}}/{mode},b=/\p{{Dia}}/i{mode},n=/\P{{Dia}}/i{mode};a.test('\u0300')&&a.test('\u0301')&&b.test('\u0300')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u0300')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Diacritic}}/{mode},s=/\s/{mode};p.test('\u0300')&&p.test('\u0301')&&!s.test('\u0300')&&!s.test('\u0301')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Dia}}K]/i{mode},b=/[^\P{{Diacritic}}K]/i{mode},c=/[\p{{Dia}}\P{{Diacritic}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u0300')&&!a.test('ſ')&&b.test('\u0301')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Dia}}$/{mode},b=/^\P{{Dia}}$/{mode};a.test('\u0300')&&!a.test('\u0301\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_diacritic_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{Dia}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uD834\uDD67');m.index===0&&m[0]==='\uD834\uDD67'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Dia}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Dia}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u0300');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Dia}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u0300');m.index===2&&m[0]==='\u0300'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Dia}}/{mode}y;r.lastIndex=1;r.exec('😀\u0300')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_diacritic_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u0301😀\u0300G'.matchAll(/(?<x>\P{{Dia}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u0301\u0300😀X'.replace(/(?<x>\p{{Diacritic}})/{mode}g,'<$1:$<x>>')==='<\u0301:\u0301><\u0300:\u0300>😀X'"
        ));
        check(&format!(
            r"'😀\u0301\u0300G'.split(/(\p{{Dia}})/{mode}).join('|')==='😀|\u0301||\u0300|G'&&'😀\u0300'.search(/\p{{Dia}}/{mode})===2&&'\u0301\u0300😀'.match(/\p{{Dia}}/{mode}g).join('')==='\u0301\u0300'"
        ));
        check(&format!(
            r"let t='';let x='\u0300\u0301😀\u0300'.replace(/(\p{{Dia}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_diacritic_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Diacritic})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Diacritic})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u0300').groups.x==='\u0300'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Dia}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Dia}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u0300')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Dia}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u0300')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_diacritic_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{Dia}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u0300'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u0300'&&m[3000]==='\\u0300'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_diacritic_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Dia}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    for source in [
        r"/[\p{Dia}&&A]/v.exec('A')",
        r"/[\p{Dia}--A]/v.exec('A')",
        r"/\p{Dia}+b/u.exec('AA')",
        r"/\p{Dia}A/v.exec('AA')",
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
fn unicode_diacritic_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{Dia}})+\B/d{mode}g,m=r.exec('😀\u0300\u0301\u0301\uFEFF');m.index===2&&m[0]==='\u0300\u0301\u0301'&&m.groups.x==='\u0301'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_diacritic_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [
        0x558u32, 0x58bu32, 0x58cu32, 0x5c8u32, 0x5c9u32, 0x656u32, 0x6e2u32, 0x8d3u32, 0xb53u32,
        0xb54u32, 0x1adeu32, 0x1adfu32, 0x1aecu32, 0x1aedu32, 0x1aeeu32, 0x1aefu32, 0x1af0u32,
        0x208fu32, 0x10ecbu32, 0x10eccu32, 0x10ecdu32, 0x10eceu32, 0x10ecfu32, 0x10ef0u32,
        0x10ef1u32, 0x10ef2u32, 0x10ef3u32, 0x10ef4u32, 0x10ef5u32, 0x10ef6u32, 0x10ef7u32,
        0x10ef9u32, 0x1d250u32, 0x1d251u32, 0x1d252u32, 0x1d25bu32, 0x1d25cu32, 0x1dfcdu32,
        0x1dfceu32, 0x1dfcfu32, 0x1dfd1u32, 0x1dfd2u32, 0x1dfd3u32, 0x1dfd4u32, 0x1dfd5u32,
        0x1dfd6u32, 0x1dfd7u32, 0x1dfd8u32, 0x1dfd9u32, 0x1dfdau32, 0x1dfdbu32, 0x1dfdcu32,
        0x1dfddu32, 0x1dfdeu32, 0x1dfdfu32, 0x1dfe0u32, 0x1dfe1u32, 0x1dfe2u32, 0x1dfe3u32,
        0x1dfe4u32, 0x1dfe5u32, 0x1dfe6u32, 0x1dfe7u32, 0x1dfe8u32, 0x1dfe9u32, 0x1dfeau32,
        0x1dfebu32, 0x1dfecu32, 0x1dfedu32, 0x1dfeeu32, 0x1dfefu32, 0x1dff0u32, 0x1dff1u32,
        0x1dff2u32, 0x1dff3u32, 0x1dff4u32, 0x1dff5u32, 0x1dff6u32, 0x1dff7u32, 0x1dff8u32,
        0x1dff9u32, 0x1dffau32, 0x1dffbu32, 0x1dffcu32, 0x1dffdu32, 0x1dffeu32, 0x1dfffu32,
    ] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Diacritic", "Dia"] {
            for flags in ["u", "ui", "v", "vi"] {
                check(&format!(
                    r"let p=new RegExp('^\\p{{{name}}}$','{flags}'),n=new RegExp('^\\P{{{name}}}$','{flags}');p.test({text:?})&&!n.test({text:?})"
                ));
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 696);
}
