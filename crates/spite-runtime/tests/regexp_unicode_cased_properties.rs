//! Cased and its exact Cased alias over complete Unicode characters.
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
fn unicode_cased_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Cased}",
        r"\P{Cased}",
        r"[^\p{Cased}]",
        r"[\p{Cased}K]",
        r"[\P{Cased}K]",
        r"^(?<x>[\p{Cased}K])$",
        r"(?<x>\P{Cased})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "\u{e0001}\u{40}\u{41}\u{5a}\u{5b}\u{60}\u{61}\u{7a}\u{7b}\u{a9}\u{aa}\u{ab}\u{b4}\u{b5}\u{b6}\u{b9}\u{ba}\u{bb}\u{bf}\u{c0}\u{d6}\u{d7}\u{d8}\u{f6}\u{f7}\u{f8}\u{1ba}\u{1bb}\u{1bc}\u{1bf}\u{1c0}\u{1c3}\u{1c4}\u{293}\u{294}\u{295}\u{296}\u{2b8}\u{2b9}\u{2bf}\u{2c0}\u{2c1}\u{2c2}\u{2df}\u{2e0}\u{2e4}\u{2e5}\u{344}\u{345}\u{346}\u{36f}\u{370}\u{373}\u{374}\u{375}\u{376}\u{377}\u{378}\u{379}\u{37a}\u{37d}\u{37e}\u{37f}\u{380}\u{385}\u{386}\u{387}\u{388}\u{38a}\u{38b}\u{38c}\u{38d}\u{38e}\u{3a1}\u{3a2}\u{3a3}\u{3f5}\u{3f6}\u{3f7}\u{481}\u{482}\u{489}\u{48a}\u{52f}\u{530}\u{531}\u{556}\u{557}\u{559}\u{55f}\u{560}\u{588}\u{589}\u{58a}\u{58d}\u{109f}\u{10a0}\u{10c5}\u{10c6}\u{10c7}\u{10c8}\u{10cc}\u{10cd}\u{10ce}\u{10cf}\u{10d0}\u{10fa}\u{10fb}\u{10fc}\u{10ff}\u{1100}\u{139f}\u{13a0}\u{13f5}\u{13f6}\u{13f7}\u{13f8}\u{13fd}\u{13fe}\u{1c7f}\u{1c80}\u{1c8a}\u{1c8b}\u{1c8f}\u{1c90}\u{1cba}\u{1cbb}\u{1cbc}\u{1cbd}\u{1cbf}\u{1cc0}\u{1cff}\u{1d00}\u{1dbf}\u{1dc0}\u{1dff}\u{1e00}\u{1f15}\u{1f16}\u{1f17}\u{1f18}\u{1f1d}\u{1f1e}\u{1f1f}\u{1f20}\u{1f45}\u{1f46}\u{1f47}\u{1f48}\u{1f4d}\u{1f4e}\u{1f4f}\u{1f50}\u{1f57}\u{1f58}\u{1f59}\u{1f5a}\u{1f5b}\u{1f5c}\u{1f5d}\u{1f5e}\u{1f5f}\u{1f7d}\u{1f7e}\u{1f7f}\u{1f80}\u{1fb4}\u{1fb5}\u{1fb6}\u{1fbc}\u{1fbd}\u{1fbe}\u{1fbf}\u{1fc1}\u{1fc2}\u{1fc4}\u{1fc5}\u{1fc6}\u{1fcc}\u{1fcd}\u{1fcf}\u{1fd0}\u{1fd3}\u{1fd4}\u{1fd5}\u{1fd6}\u{1fdb}\u{1fdc}\u{1fdf}\u{1fe0}\u{1fec}\u{1fed}\u{1ff1}\u{1ff2}\u{1ff4}\u{1ff5}\u{1ff6}\u{1ffc}\u{1ffd}\u{2070}\u{2071}\u{2072}\u{207e}\u{207f}\u{2080}\u{208f}\u{2090}\u{20a0}\u{2101}\u{2102}\u{2103}\u{2106}\u{2107}\u{2108}\u{2109}\u{210a}\u{2113}\u{2114}\u{2115}\u{2116}\u{2118}\u{2119}\u{211d}\u{211e}\u{2123}\u{2124}\u{2125}\u{2126}\u{2127}\u{2128}\u{2129}\u{212a}\u{212d}\u{212e}\u{212f}\u{2134}\u{2135}\u{2138}\u{2139}\u{213a}\u{213b}\u{213c}\u{213f}\u{2140}\u{2144}\u{2145}\u{2149}\u{214a}\u{214d}\u{214e}\u{214f}\u{215f}\u{2160}\u{217f}\u{2180}\u{2182}\u{2183}\u{2184}\u{2185}\u{24b5}\u{24b6}\u{24e9}\u{24ea}\u{2bff}\u{2c00}\u{2ce4}\u{2ce5}\u{2cea}\u{2ceb}\u{2cee}\u{2cef}\u{2cf1}\u{2cf2}\u{2cf3}\u{2cf4}\u{2cff}\u{2d00}\u{2d25}\u{2d26}\u{2d27}\u{2d28}\u{2d2c}\u{2d2d}\u{2d2e}\u{a63f}\u{a640}\u{a66d}\u{a66e}\u{a67f}\u{a680}\u{a69d}\u{a69e}\u{a721}\u{a722}\u{a787}\u{a788}\u{a78a}\u{a78b}\u{a78e}\u{a78f}\u{a790}\u{a7de}\u{a7e1}\u{a7e3}\u{a7f0}\u{a7f1}\u{a7f6}\u{a7f7}\u{a7f8}\u{a7fa}\u{a7fb}\u{ab2f}\u{ab30}\u{ab5a}\u{ab5b}\u{ab5c}\u{ab69}\u{ab6a}\u{ab6b}\u{ab6e}\u{ab6f}\u{ab70}\u{abbf}\u{abc0}\u{faff}\u{fb00}\u{fb06}\u{fb07}\u{fb12}\u{fb13}\u{fb17}\u{fb18}\u{ff20}\u{ff21}\u{ff3a}\u{ff3b}\u{ff40}\u{ff41}\u{ff5a}\u{ff5b}\u{103ff}\u{10400}\u{1044f}\u{10450}\u{104af}\u{104b0}\u{104d3}\u{104d4}\u{104d7}\u{104d8}\u{104fb}\u{104fc}\u{1056f}\u{10570}\u{1057a}\u{1057b}\u{1057c}\u{1058a}\u{1058b}\u{1058c}\u{10592}\u{10593}\u{10594}\u{10595}\u{10596}\u{10597}\u{105a1}\u{105a2}\u{105a3}\u{105b1}\u{105b2}\u{105b3}\u{105b9}\u{105ba}\u{105bb}\u{105bc}\u{105bd}\u{1077f}\u{10780}\u{10781}\u{10782}\u{10783}\u{10785}\u{10786}\u{10787}\u{107b0}\u{107b1}\u{107b2}\u{107c0}\u{10c7f}\u{10c80}\u{10cb2}\u{10cb3}\u{10cbf}\u{10cc0}\u{10cf2}\u{10cf3}\u{10d4f}\u{10d50}\u{10d65}\u{10d66}\u{10d6f}\u{10d70}\u{10d85}\u{10d86}\u{1189f}\u{118a0}\u{118df}\u{118e0}\u{16e3f}\u{16e40}\u{16e7f}\u{16e80}\u{16e9f}\u{16ea0}\u{16eb8}\u{16eb9}\u{16eba}\u{16ebb}\u{16ed3}\u{16ed4}\u{1d3ff}\u{1d400}\u{1d454}\u{1d455}\u{1d456}\u{1d49c}\u{1d49d}\u{1d49e}\u{1d49f}\u{1d4a0}\u{1d4a1}\u{1d4a2}\u{1d4a3}\u{1d4a4}\u{1d4a5}\u{1d4a6}\u{1d4a7}\u{1d4a8}\u{1d4a9}\u{1d4ac}\u{1d4ad}\u{1d4ae}\u{1d4b9}\u{1d4ba}\u{1d4bb}\u{1d4bc}\u{1d4bd}\u{1d4c3}\u{1d4c4}\u{1d4c5}\u{1d505}\u{1d506}\u{1d507}\u{1d50a}\u{1d50b}\u{1d50c}\u{1d50d}\u{1d514}\u{1d515}\u{1d516}\u{1d51c}\u{1d51d}\u{1d51e}\u{1d539}\u{1d53a}\u{1d53b}\u{1d53e}\u{1d53f}\u{1d540}\u{1d544}\u{1d545}\u{1d546}\u{1d547}\u{1d549}\u{1d54a}\u{1d550}\u{1d551}\u{1d552}\u{1d6a7}\u{1d6a8}\u{1d6c0}\u{1d6c1}\u{1d6c2}\u{1d6da}\u{1d6db}\u{1d6dc}\u{1d6fa}\u{1d6fb}\u{1d6fc}\u{1d714}\u{1d715}\u{1d716}\u{1d734}\u{1d735}\u{1d736}\u{1d74e}\u{1d74f}\u{1d750}\u{1d76e}\u{1d76f}\u{1d770}\u{1d788}\u{1d789}\u{1d78a}\u{1d7a8}\u{1d7a9}\u{1d7aa}\u{1d7c2}\u{1d7c3}\u{1d7c4}\u{1d7cb}\u{1d7cc}\u{1deff}\u{1df00}\u{1df09}\u{1df0a}\u{1df0b}\u{1df80}\u{1df8f}\u{1df97}\u{1dfcc}\u{1e000}\u{1e02f}\u{1e030}\u{1e06d}\u{1e06e}\u{1e8ff}\u{1e900}\u{1e943}\u{1e944}\u{1f12f}\u{1f130}\u{1f149}\u{1f14a}\u{1f14f}\u{1f150}\u{1f169}\u{1f16a}\u{1f16f}\u{1f170}\u{1f189}\u{1f18a}\u{feff}ſK\n",
                "X\r\n\u{41}\u{2028}\u{42}\u{2029}",
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
fn unicode_cased_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Cased}}/{mode},b=/\p{{Cased}}/i{mode},n=/\P{{Cased}}/i{mode};a.test('\u0041')&&a.test('\u0042')&&b.test('\u0041')&&!a.test('\u200D')&&!b.test('\u005f')&&b.test('ſ')&&b.test('K')&&!n.test('\u0041')&&n.test('\uDB40\uDC01')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Cased}}/{mode},s=/\s/{mode};p.test('\u0041')&&p.test('\u0042')&&!s.test('\u0041')&&!s.test('\u0042')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Cased}}K]/i{mode},b=/[^\P{{Cased}}K]/i{mode},c=/[\p{{Cased}}\P{{Cased}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u0041')&&a.test('ſ')&&b.test('\u0042')&&!b.test('K')&&c.test('\uDB40\uDC01')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Cased}}$/{mode},b=/^\P{{Cased}}$/{mode};a.test('\u0041')&&!a.test('\u0042\n')&&!a.test('')&&!a.test('\n')&&b.test('\uDB40\uDC01')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_cased_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Cased}}K]))$/dim{mode}g,m=r.exec('\u200D\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Cased}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u0041');m.index===0&&m[1]==='\uDB40\uDC01'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Cased}}/d{mode}g;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u0041');m.index===2&&m[0]==='\u0041'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Cased}}/{mode}y;r.lastIndex=1;r.exec('\uDB40\uDC01\u0041')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_cased_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u0042\uDB40\uDC01\u0041\u200D'.matchAll(/(?<x>\P{{Cased}})/d{mode}g)];a.length===2&&a[0][1]==='\uDB40\uDC01'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='\u200D'"
        ));
        check(&format!(
            r"'\u0042\u0041\uDB40\uDC01\u200D'.replace(/(?<x>\p{{Cased}})/{mode}g,'<$1:$<x>>')==='<\u0042:\u0042><\u0041:\u0041>\uDB40\uDC01\u200D'"
        ));
        check(&format!(
            r"'\uDB40\uDC01\u0042\u0041\u200D'.split(/(\p{{Cased}})/{mode}).join('|')==='\uDB40\uDC01|\u0042||\u0041|\u200D'&&'\uDB40\uDC01\u0041'.search(/\p{{Cased}}/{mode})===2&&'\u0042\u0041\uDB40\uDC01'.match(/\p{{Cased}}/{mode}g).join('')==='\u0042\u0041'"
        ));
        check(&format!(
            r"let t='';let x='\u0041\u0042\uDB40\uDC01\u0041'.replace(/(\p{{Cased}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__\uDB40\uDC01_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_cased_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Cased})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Cased})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u0041').groups.x==='\u0041'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Cased}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '\uDB40\uDC01'}});t==='si'&&m[0]==='\uDB40\uDC01'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Cased}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u0041')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Cased}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\uDB40\uDC01\u0041')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_cased_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{Cased}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='\\uDB40\\uDC01'.repeat(100000)+'\\n\\u0041'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u0041'&&m[3000]==='\\u0041'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_cased_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Cased}/ug,marker=0,text='\\uDB40\\uDC01'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    for source in [
        r"/\p{Script_Extensions=Latin}/u.exec('A')",
        r"/[\p{Cased}&&A]/v.exec('A')",
        r"/[\p{Cased}--A]/v.exec('A')",
        r"/\p{Cased}+b/u.exec('AA')",
        r"/\p{Cased}A/v.exec('AA')",
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
fn unicode_cased_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\b(?<x>\p{{Cased}})+\b/d{mode}g,m=r.exec('\uDB40\uDC01\u0041\u0042\u0042\uFEFF');m.index===2&&m[0]==='\u0041\u0042\u0042'&&m.groups.x==='\u0042'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_cased_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [
        0x558u32, 0x58bu32, 0x58cu32, 0x209du32, 0x209eu32, 0x209fu32, 0xa7ddu32, 0xa7e2u32,
        0xab6cu32, 0xab6du32, 0x107bbu32, 0x107bcu32, 0x107bdu32, 0x107beu32, 0x107bfu32,
        0x1d6a6u32, 0x1df1fu32, 0x1df20u32, 0x1df21u32, 0x1df22u32, 0x1df23u32, 0x1df24u32,
        0x1df2bu32, 0x1df2cu32, 0x1df2du32, 0x1df2eu32, 0x1df2fu32, 0x1df30u32, 0x1df31u32,
        0x1df32u32, 0x1df33u32, 0x1df34u32, 0x1df35u32, 0x1df36u32, 0x1df37u32, 0x1df38u32,
        0x1df39u32, 0x1df3au32, 0x1df3bu32, 0x1df3cu32, 0x1df3du32, 0x1df3eu32, 0x1df3fu32,
        0x1df40u32, 0x1df41u32, 0x1df42u32, 0x1df43u32, 0x1df44u32, 0x1df45u32, 0x1df46u32,
        0x1df47u32, 0x1df48u32, 0x1df49u32, 0x1df4au32, 0x1df4bu32, 0x1df4cu32, 0x1df4du32,
        0x1df4eu32, 0x1df4fu32, 0x1df50u32, 0x1df51u32, 0x1df52u32, 0x1df53u32, 0x1df54u32,
        0x1df55u32, 0x1df56u32, 0x1df57u32, 0x1df58u32, 0x1df59u32, 0x1df5au32, 0x1df5bu32,
        0x1df5cu32, 0x1df5du32, 0x1df5eu32, 0x1df5fu32, 0x1df60u32, 0x1df61u32, 0x1df62u32,
        0x1df63u32, 0x1df64u32, 0x1df65u32, 0x1df66u32, 0x1df67u32, 0x1df68u32, 0x1df69u32,
        0x1df6au32, 0x1df6bu32, 0x1df6cu32, 0x1df6du32, 0x1df6eu32, 0x1df6fu32, 0x1df70u32,
        0x1df71u32, 0x1df72u32, 0x1df73u32, 0x1df74u32, 0x1df75u32, 0x1df76u32, 0x1df77u32,
        0x1df78u32, 0x1df79u32, 0x1df7au32, 0x1df7bu32, 0x1df7cu32, 0x1df7du32, 0x1df7eu32,
        0x1df7fu32, 0x1df90u32, 0x1df91u32, 0x1df92u32, 0x1df93u32, 0x1df94u32, 0x1df95u32,
        0x1df96u32, 0x1dfcdu32, 0x1dfceu32, 0x1dfcfu32, 0x1dfd0u32, 0x1dfd1u32, 0x1dfd2u32,
        0x1dfd3u32, 0x1dfd4u32, 0x1dfd5u32, 0x1dfd6u32, 0x1dfd7u32, 0x1dfd8u32, 0x1dfd9u32,
        0x1dfdau32, 0x1dfdbu32, 0x1dfdcu32, 0x1dfddu32, 0x1dfdeu32, 0x1dfdfu32, 0x1dfe0u32,
        0x1dfe1u32, 0x1dfe2u32, 0x1dfe3u32, 0x1dfe4u32, 0x1dfe5u32, 0x1dfe6u32, 0x1dfe7u32,
        0x1dfe8u32, 0x1dfe9u32, 0x1dfeau32, 0x1dfebu32, 0x1dfecu32, 0x1dfedu32, 0x1dfeeu32,
        0x1dfefu32, 0x1dff0u32, 0x1dff1u32, 0x1dff2u32, 0x1dff3u32, 0x1dff4u32, 0x1dff5u32,
        0x1dff6u32, 0x1dff7u32, 0x1dff8u32, 0x1dff9u32, 0x1dffau32, 0x1dffbu32, 0x1dffcu32,
        0x1dffdu32, 0x1dffeu32, 0x1dfffu32,
    ] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for flags in ["u", "ui", "v", "vi"] {
            check(&format!(
                r"let p=new RegExp('^\\p{{Cased}}$','{flags}'),n=new RegExp('^\\P{{Cased}}$','{flags}');p.test({text:?})&&!n.test({text:?})"
            ));
            comparisons += 1;
        }
    }
    assert_eq!(comparisons, 660);
}
