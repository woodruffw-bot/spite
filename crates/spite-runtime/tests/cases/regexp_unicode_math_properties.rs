//! Math and its exact Math alias over complete Unicode characters.
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
fn unicode_math_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Math}",
        r"\P{Math}",
        r"[^\p{Math}]",
        r"[\p{Math}K]",
        r"[\P{Math}K]",
        r"^(?<x>[\p{Math}K])$",
        r"(?<x>\P{Math})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "\u{e0001}\u{2a}\u{2b}\u{2c}\u{3b}\u{3c}\u{3e}\u{3f}\u{5d}\u{5e}\u{5f}\u{7b}\u{7c}\u{7d}\u{7e}\u{7f}\u{ab}\u{ac}\u{ad}\u{b0}\u{b1}\u{b2}\u{d6}\u{d7}\u{d8}\u{f6}\u{f7}\u{f8}\u{392}\u{395}\u{398}\u{39a}\u{3a1}\u{3a6}\u{3b2}\u{3b5}\u{3b8}\u{3ba}\u{3c1}\u{3c6}\u{3cf}\u{3d0}\u{3d1}\u{3d2}\u{3d3}\u{3d4}\u{3d5}\u{3d6}\u{3ef}\u{3f0}\u{3f1}\u{3f2}\u{3f3}\u{3f4}\u{3f5}\u{3f6}\u{3f7}\u{605}\u{606}\u{608}\u{609}\u{2015}\u{2016}\u{2017}\u{2031}\u{2032}\u{2034}\u{2035}\u{203f}\u{2040}\u{2041}\u{2043}\u{2044}\u{2045}\u{2051}\u{2052}\u{2053}\u{2060}\u{2061}\u{2064}\u{2065}\u{2079}\u{207a}\u{207e}\u{207f}\u{2089}\u{208a}\u{208e}\u{208f}\u{20cf}\u{20d0}\u{20dc}\u{20dd}\u{20e0}\u{20e1}\u{20e2}\u{20e4}\u{20e5}\u{20e6}\u{20e7}\u{20ea}\u{20eb}\u{20ef}\u{20f0}\u{2101}\u{2102}\u{2103}\u{2106}\u{2107}\u{2108}\u{2109}\u{210a}\u{2113}\u{2114}\u{2115}\u{2116}\u{2117}\u{2118}\u{211d}\u{211e}\u{2123}\u{2124}\u{2125}\u{2127}\u{2128}\u{2129}\u{212a}\u{212b}\u{212c}\u{212d}\u{212e}\u{212f}\u{2131}\u{2132}\u{2133}\u{2138}\u{2139}\u{213b}\u{213c}\u{2149}\u{214a}\u{214b}\u{214c}\u{218f}\u{2190}\u{21a7}\u{21a8}\u{21a9}\u{21ae}\u{21af}\u{21b0}\u{21b1}\u{21b2}\u{21b5}\u{21b6}\u{21b7}\u{21b8}\u{21bb}\u{21bc}\u{21db}\u{21dc}\u{21dd}\u{21de}\u{21e3}\u{21e4}\u{21e5}\u{21e6}\u{21f3}\u{21f4}\u{22ff}\u{2300}\u{2307}\u{2308}\u{230b}\u{230c}\u{231f}\u{2320}\u{2321}\u{2322}\u{237b}\u{237c}\u{237d}\u{239a}\u{239b}\u{23b5}\u{23b6}\u{23b7}\u{23b8}\u{23cf}\u{23d0}\u{23d1}\u{23db}\u{23dc}\u{23e2}\u{23e3}\u{259f}\u{25a0}\u{25a1}\u{25a2}\u{25ad}\u{25ae}\u{25b7}\u{25b8}\u{25bb}\u{25bc}\u{25c1}\u{25c2}\u{25c5}\u{25c6}\u{25c7}\u{25c8}\u{25c9}\u{25ca}\u{25cb}\u{25cc}\u{25ce}\u{25cf}\u{25d3}\u{25d4}\u{25e1}\u{25e2}\u{25e3}\u{25e4}\u{25e5}\u{25e6}\u{25e7}\u{25ec}\u{25ed}\u{25f7}\u{25f8}\u{25ff}\u{2600}\u{2604}\u{2605}\u{2606}\u{2607}\u{263f}\u{2640}\u{2641}\u{2642}\u{2643}\u{265f}\u{2660}\u{2663}\u{2664}\u{266c}\u{266d}\u{266f}\u{2670}\u{27bf}\u{27c0}\u{27ff}\u{2800}\u{28ff}\u{2900}\u{2aff}\u{2b00}\u{2b2f}\u{2b30}\u{2b44}\u{2b45}\u{2b46}\u{2b47}\u{2b4c}\u{2b4d}\u{fb28}\u{fb29}\u{fb2a}\u{fe60}\u{fe61}\u{fe66}\u{fe67}\u{fe68}\u{fe69}\u{ff0a}\u{ff0b}\u{ff0c}\u{ff1b}\u{ff1c}\u{ff1e}\u{ff1f}\u{ff3b}\u{ff3c}\u{ff3d}\u{ff3e}\u{ff3f}\u{ff5b}\u{ff5c}\u{ff5d}\u{ff5e}\u{ff5f}\u{ffe1}\u{ffe2}\u{ffe3}\u{ffe8}\u{ffe9}\u{ffec}\u{ffed}\u{10d8d}\u{10d8e}\u{10d8f}\u{10d90}\u{1cedc}\u{1cee0}\u{1ceef}\u{1cef0}\u{1cefe}\u{1d3ff}\u{1d400}\u{1d454}\u{1d455}\u{1d456}\u{1d49c}\u{1d49d}\u{1d49e}\u{1d49f}\u{1d4a0}\u{1d4a1}\u{1d4a2}\u{1d4a3}\u{1d4a4}\u{1d4a5}\u{1d4a6}\u{1d4a7}\u{1d4a8}\u{1d4a9}\u{1d4ac}\u{1d4ad}\u{1d4ae}\u{1d4b9}\u{1d4ba}\u{1d4bb}\u{1d4bc}\u{1d4bd}\u{1d4c3}\u{1d4c4}\u{1d4c5}\u{1d505}\u{1d506}\u{1d507}\u{1d50a}\u{1d50b}\u{1d50c}\u{1d50d}\u{1d514}\u{1d515}\u{1d516}\u{1d51c}\u{1d51d}\u{1d51e}\u{1d539}\u{1d53a}\u{1d53b}\u{1d53e}\u{1d53f}\u{1d540}\u{1d544}\u{1d545}\u{1d546}\u{1d547}\u{1d549}\u{1d54a}\u{1d550}\u{1d551}\u{1d552}\u{1d6a7}\u{1d6a8}\u{1d7cb}\u{1d7cc}\u{1d7cd}\u{1d7ce}\u{1d7ff}\u{1d800}\u{1daff}\u{1db1d}\u{1edff}\u{1ee00}\u{1ee03}\u{1ee04}\u{1ee05}\u{1ee1f}\u{1ee20}\u{1ee21}\u{1ee22}\u{1ee23}\u{1ee24}\u{1ee25}\u{1ee26}\u{1ee27}\u{1ee28}\u{1ee29}\u{1ee32}\u{1ee33}\u{1ee34}\u{1ee37}\u{1ee38}\u{1ee39}\u{1ee3a}\u{1ee3b}\u{1ee3c}\u{1ee41}\u{1ee42}\u{1ee43}\u{1ee46}\u{1ee47}\u{1ee48}\u{1ee49}\u{1ee4a}\u{1ee4b}\u{1ee4c}\u{1ee4d}\u{1ee4f}\u{1ee50}\u{1ee51}\u{1ee52}\u{1ee53}\u{1ee54}\u{1ee55}\u{1ee56}\u{1ee57}\u{1ee58}\u{1ee59}\u{1ee5a}\u{1ee5b}\u{1ee5c}\u{1ee5d}\u{1ee5e}\u{1ee5f}\u{1ee60}\u{1ee61}\u{1ee62}\u{1ee63}\u{1ee64}\u{1ee65}\u{1ee66}\u{1ee67}\u{1ee6a}\u{1ee6b}\u{1ee6c}\u{1ee72}\u{1ee73}\u{1ee74}\u{1ee77}\u{1ee78}\u{1ee79}\u{1ee7c}\u{1ee7d}\u{1ee7e}\u{1ee7f}\u{1ee80}\u{1ee89}\u{1ee8a}\u{1ee8b}\u{1ee9b}\u{1ee9c}\u{1eea0}\u{1eea1}\u{1eea3}\u{1eea4}\u{1eea5}\u{1eea9}\u{1eeaa}\u{1eeab}\u{1eebb}\u{1eebc}\u{1eeef}\u{1eef0}\u{1eef1}\u{1eef2}\u{1f7da}\u{1f7dc}\u{1f7f0}\u{1f800}\u{1f8cf}\u{1f8d0}\u{1f8d8}\u{1f8d9}\u{feff}ſK\n",
                "X\r\n\u{2b}\u{2028}\u{d7}\u{2029}",
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
fn unicode_math_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let p=/^\p{{Math}}$/i{mode},n=/^\P{{Math}}$/i{mode};'\u0392\u0395\u0398\u039A\u03A1\u03A6\u03B2\u03B5\u03B8\u03BA\u03C1\u03C6\u03D0\u03D1\u03D5\u03F0\u03F1\u03F4\u03F5'.split('').every(c=>p.test(c)&&n.test(c)===('{mode}'==='u'))"
        ));
        check(&format!(
            r"let p=/[\p{{Math}}K]/i{mode},n=/[\P{{Math}}K]/i{mode},x=/[^\P{{Math}}K]/i{mode};p.test('K')&&n.test('K')&&!x.test('K')&&'\u0392\u0395\u0398\u039A\u03A1\u03A6\u03B2\u03B5\u03B8\u03BA\u03C1\u03C6\u03D0\u03D1\u03D5\u03F0\u03F1\u03F4\u03F5'.split('').every(c=>p.test(c)&&n.test(c)===('{mode}'==='u')&&x.test(c)===('{mode}'==='v'))"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Math}}/{mode},b=/\p{{Math}}/i{mode},n=/\P{{Math}}/i{mode};a.test('\u002B')&&a.test('\u00D7')&&b.test('\u002B')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u002B')&&n.test('\uDB40\uDC01')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Math}}/{mode},s=/\s/{mode};p.test('\u002B')&&p.test('\u00D7')&&!s.test('\u002B')&&!s.test('\u00D7')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Math}}K]/i{mode},b=/[^\P{{Math}}K]/i{mode},c=/[\p{{Math}}\P{{Math}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u002B')&&!a.test('ſ')&&b.test('\u00D7')&&!b.test('K')&&c.test('\uDB40\uDC01')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Math}}$/{mode},b=/^\P{{Math}}$/{mode};a.test('\u002B')&&!a.test('\u00D7\n')&&!a.test('')&&!a.test('\n')&&b.test('\uDB40\uDC01')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_math_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{Math}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uD835\uDC00');m.index===0&&m[0]==='\uD835\uDC00'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Math}}K]))$/dim{mode}g,m=r.exec('\u200D\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Math}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u002B');m.index===0&&m[1]==='\uDB40\uDC01'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Math}}/d{mode}g;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u002B');m.index===2&&m[0]==='\u002B'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Math}}/{mode}y;r.lastIndex=1;r.exec('\uDB40\uDC01\u002B')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_math_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u00D7\uDB40\uDC01\u002B\u200D'.matchAll(/(?<x>\P{{Math}})/d{mode}g)];a.length===2&&a[0][1]==='\uDB40\uDC01'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='\u200D'"
        ));
        check(&format!(
            r"'\u00D7\u002B\uDB40\uDC01\u200D'.replace(/(?<x>\p{{Math}})/{mode}g,'<$1:$<x>>')==='<\u00D7:\u00D7><\u002B:\u002B>\uDB40\uDC01\u200D'"
        ));
        check(&format!(
            r"'\uDB40\uDC01\u00D7\u002B\u200D'.split(/(\p{{Math}})/{mode}).join('|')==='\uDB40\uDC01|\u00D7||\u002B|\u200D'&&'\uDB40\uDC01\u002B'.search(/\p{{Math}}/{mode})===2&&'\u00D7\u002B\uDB40\uDC01'.match(/\p{{Math}}/{mode}g).join('')==='\u00D7\u002B'"
        ));
        check(&format!(
            r"let t='';let x='\u002B\u00D7\uDB40\uDC01\u002B'.replace(/(\p{{Math}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__\uDB40\uDC01_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_math_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Math})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Math})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u002B').groups.x==='\u002B'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Math}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '\uDB40\uDC01'}});t==='si'&&m[0]==='\uDB40\uDC01'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Math}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u002B')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Math}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\uDB40\uDC01\u002B')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_math_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{Math}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='\\uDB40\\uDC01'.repeat(100000)+'\\n\\u002B'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u002B'&&m[3000]==='\\u002B'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_math_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Math}/ug,marker=0,text='\\uDB40\\uDC01'.repeat(10000);r.lastIndex=1")
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
        r"/[\p{Math}&&A]/v.exec('A')",
        r"/[\p{Math}--A]/v.exec('A')",
        r"/\p{Math}+b/u.exec('AA')",
        r"/\p{Math}A/v.exec('AA')",
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
fn unicode_math_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{Math}})+\B/d{mode}g,m=r.exec('\uDB40\uDC01\u002B\u00D7\u00D7\uFEFF');m.index===2&&m[0]==='\u002B\u00D7\u00D7'&&m.groups.x==='\u00D7'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_math_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [
        0x1ceddu32, 0x1cedeu32, 0x1cedfu32, 0x1cef1u32, 0x1cef7u32, 0x1cefdu32, 0x1d6a6u32,
        0x1db00u32, 0x1db0eu32, 0x1db1cu32, 0x1f7dbu32, 0x1f7f1u32, 0x1f7f8u32, 0x1f7ffu32,
    ] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Math"] {
            for flags in ["u", "ui", "v", "vi"] {
                check(&format!(
                    r"let p=new RegExp('^\\p{{{name}}}$','{flags}'),n=new RegExp('^\\P{{{name}}}$','{flags}');p.test({text:?})&&!n.test({text:?})"
                ));
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 56);
}
