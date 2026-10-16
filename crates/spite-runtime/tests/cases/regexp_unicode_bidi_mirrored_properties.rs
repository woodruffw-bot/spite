//! Bidi_Mirrored and its exact Bidi_M alias over complete Unicode characters.
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
fn unicode_bidi_mirrored_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Bidi_Mirrored}",
        r"\P{Bidi_M}",
        r"[^\p{Bidi_M}]",
        r"[\p{Bidi_M}K]",
        r"[\P{Bidi_Mirrored}K]",
        r"^(?<x>[\p{Bidi_M}K])$",
        r"(?<x>\P{Bidi_Mirrored})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{27}\u{28}\u{29}\u{2a}\u{3b}\u{3c}\u{3d}\u{3e}\u{3f}\u{5a}\u{5b}\u{5c}\u{5d}\u{5e}\u{7a}\u{7b}\u{7c}\u{7d}\u{7e}\u{aa}\u{ab}\u{ac}\u{ba}\u{bb}\u{bc}\u{f39}\u{f3a}\u{f3d}\u{f3e}\u{169a}\u{169b}\u{169c}\u{169d}\u{2038}\u{2039}\u{203a}\u{203b}\u{2044}\u{2045}\u{2046}\u{2047}\u{207c}\u{207d}\u{207e}\u{207f}\u{208c}\u{208d}\u{208e}\u{208f}\u{213f}\u{2140}\u{2141}\u{2200}\u{2201}\u{2204}\u{2205}\u{2207}\u{2208}\u{220d}\u{220e}\u{2210}\u{2211}\u{2212}\u{2214}\u{2215}\u{2216}\u{2217}\u{2219}\u{221a}\u{221d}\u{221e}\u{221f}\u{2222}\u{2223}\u{2224}\u{2225}\u{2226}\u{2227}\u{222a}\u{222b}\u{2233}\u{2234}\u{2238}\u{2239}\u{223a}\u{223b}\u{224c}\u{224d}\u{2251}\u{2252}\u{2255}\u{2256}\u{225e}\u{225f}\u{2260}\u{2261}\u{2262}\u{2263}\u{2264}\u{226b}\u{226c}\u{226d}\u{228c}\u{228d}\u{228e}\u{228f}\u{2292}\u{2293}\u{2297}\u{2298}\u{2299}\u{22a1}\u{22a2}\u{22a3}\u{22a4}\u{22a5}\u{22a6}\u{22b8}\u{22b9}\u{22bd}\u{22be}\u{22bf}\u{22c0}\u{22c8}\u{22c9}\u{22cd}\u{22ce}\u{22cf}\u{22d0}\u{22d1}\u{22d2}\u{22d5}\u{22d6}\u{22ed}\u{22ee}\u{22ef}\u{22f0}\u{22ff}\u{2300}\u{2307}\u{2308}\u{230b}\u{230c}\u{231f}\u{2320}\u{2321}\u{2322}\u{2328}\u{2329}\u{232a}\u{232b}\u{2767}\u{2768}\u{2775}\u{2776}\u{27bf}\u{27c0}\u{27c1}\u{27c2}\u{27c3}\u{27c6}\u{27c7}\u{27c8}\u{27c9}\u{27ca}\u{27cb}\u{27cd}\u{27ce}\u{27d2}\u{27d3}\u{27d6}\u{27d7}\u{27db}\u{27dc}\u{27de}\u{27df}\u{27e1}\u{27e2}\u{27ef}\u{27f0}\u{2982}\u{2983}\u{2998}\u{2999}\u{299a}\u{299b}\u{29a0}\u{29a1}\u{29a2}\u{29af}\u{29b0}\u{29b7}\u{29b8}\u{29b9}\u{29bf}\u{29c0}\u{29c5}\u{29c6}\u{29c8}\u{29c9}\u{29ca}\u{29cd}\u{29ce}\u{29d2}\u{29d3}\u{29d4}\u{29d5}\u{29d6}\u{29d7}\u{29d8}\u{29dc}\u{29dd}\u{29e0}\u{29e1}\u{29e2}\u{29e3}\u{29e5}\u{29e6}\u{29e7}\u{29e8}\u{29e9}\u{29ea}\u{29f3}\u{29f4}\u{29f9}\u{29fa}\u{29fb}\u{29fc}\u{29fd}\u{29fe}\u{2a09}\u{2a0a}\u{2a1c}\u{2a1d}\u{2a1e}\u{2a21}\u{2a22}\u{2a23}\u{2a24}\u{2a25}\u{2a26}\u{2a27}\u{2a28}\u{2a29}\u{2a2a}\u{2a2b}\u{2a2e}\u{2a2f}\u{2a33}\u{2a34}\u{2a35}\u{2a36}\u{2a3b}\u{2a3c}\u{2a3e}\u{2a3f}\u{2a56}\u{2a57}\u{2a58}\u{2a59}\u{2a63}\u{2a64}\u{2a65}\u{2a66}\u{2a69}\u{2a6a}\u{2a6d}\u{2a6e}\u{2a6f}\u{2a70}\u{2a71}\u{2a72}\u{2a73}\u{2a74}\u{2a75}\u{2a78}\u{2a79}\u{2aa3}\u{2aa4}\u{2aa5}\u{2aa6}\u{2aad}\u{2aae}\u{2aaf}\u{2ad6}\u{2ad7}\u{2adb}\u{2adc}\u{2add}\u{2ade}\u{2adf}\u{2ae1}\u{2ae2}\u{2ae6}\u{2ae7}\u{2aeb}\u{2aec}\u{2aee}\u{2aef}\u{2af2}\u{2af3}\u{2af4}\u{2af6}\u{2af7}\u{2afb}\u{2afc}\u{2afd}\u{2afe}\u{2bfd}\u{2bfe}\u{2bff}\u{2e01}\u{2e02}\u{2e05}\u{2e06}\u{2e08}\u{2e09}\u{2e0a}\u{2e0b}\u{2e0c}\u{2e0d}\u{2e0e}\u{2e1b}\u{2e1c}\u{2e1d}\u{2e1e}\u{2e1f}\u{2e20}\u{2e29}\u{2e2a}\u{2e54}\u{2e55}\u{2e5c}\u{2e5d}\u{2e61}\u{2e64}\u{3007}\u{3008}\u{3011}\u{3012}\u{3013}\u{3014}\u{301b}\u{301c}\u{fe58}\u{fe59}\u{fe5e}\u{fe5f}\u{fe63}\u{fe64}\u{fe65}\u{fe66}\u{ff07}\u{ff08}\u{ff09}\u{ff0a}\u{ff1b}\u{ff1c}\u{ff1d}\u{ff1e}\u{ff1f}\u{ff3a}\u{ff3b}\u{ff3c}\u{ff3d}\u{ff3e}\u{ff5a}\u{ff5b}\u{ff5c}\u{ff5d}\u{ff5e}\u{ff5f}\u{ff60}\u{ff61}\u{ff62}\u{ff63}\u{ff64}\u{1cedc}\u{1cee0}\u{1d6da}\u{1d6db}\u{1d6dc}\u{1d714}\u{1d715}\u{1d716}\u{1d74e}\u{1d74f}\u{1d750}\u{1d788}\u{1d789}\u{1d78a}\u{1d7c2}\u{1d7c3}\u{1d7c4}\u{1db02}\u{1db0a}\u{1db0f}\u{1db11}\u{1db16}\u{1db19}\u{1db1a}\u{1db1c}\u{feff}ſK\n",
                "X\r\n\u{28}\u{2028}\u{ab}\u{2029}",
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
fn unicode_bidi_mirrored_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Bidi_Mirrored}}/{mode},b=/\p{{Bidi_M}}/i{mode},n=/\P{{Bidi_M}}/i{mode};a.test('\u0028')&&a.test('\u0029')&&b.test('\u0028')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u0028')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Bidi_Mirrored}}/{mode},s=/\s/{mode};p.test('\u0028')&&p.test('\u0029')&&!s.test('\u0028')&&!s.test('\u0029')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Bidi_M}}K]/i{mode},b=/[^\P{{Bidi_Mirrored}}K]/i{mode},c=/[\p{{Bidi_M}}\P{{Bidi_Mirrored}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u0028')&&!a.test('ſ')&&b.test('\u0029')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Bidi_M}}$/{mode},b=/^\P{{Bidi_M}}$/{mode};a.test('\u0028')&&!a.test('\u0029\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_bidi_mirrored_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Bidi_M}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Bidi_M}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u0028');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Bidi_M}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u0028');m.index===2&&m[0]==='\u0028'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Bidi_M}}/{mode}y;r.lastIndex=1;r.exec('😀\u0028')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_bidi_mirrored_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u0029😀\u0028G'.matchAll(/(?<x>\P{{Bidi_M}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u0029\u0028😀X'.replace(/(?<x>\p{{Bidi_Mirrored}})/{mode}g,'<$1:$<x>>')==='<\u0029:\u0029><\u0028:\u0028>😀X'"
        ));
        check(&format!(
            r"'😀\u0029\u0028G'.split(/(\p{{Bidi_M}})/{mode}).join('|')==='😀|\u0029||\u0028|G'&&'😀\u0028'.search(/\p{{Bidi_M}}/{mode})===2&&'\u0029\u0028😀'.match(/\p{{Bidi_M}}/{mode}g).join('')==='\u0029\u0028'"
        ));
        check(&format!(
            r"let t='';let x='\u0028\u0029😀\u0028'.replace(/(\p{{Bidi_M}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_bidi_mirrored_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Bidi_Mirrored})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Bidi_Mirrored})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u0028').groups.x==='\u0028'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Bidi_M}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Bidi_M}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u0028')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Bidi_M}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u0028')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_bidi_mirrored_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{Bidi_M}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u0028'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u0028'&&m[3000]==='\\u0028'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_bidi_mirrored_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Bidi_M}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    check(r"/[\p{Bidi_M}&&A]/v.exec('A')===null");
    check(r"/[\p{Bidi_M}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x28),m=/[\p{Bidi_M}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [r"/\p{Bidi_M}+b/u.exec('AA')", r"/\p{Bidi_M}A/v.exec('AA')"] {
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
fn unicode_bidi_mirrored_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{Bidi_M}})+\B/d{mode}g,m=r.exec('😀\u0028\u0029\u00AB!');m.index===2&&m[0]==='\u0028\u0029\u00AB'&&m.groups.x==='\u00AB'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_bidi_mirrored_property_unicode18_additions() {
    let mut comparisons = 0;
    for point in [
        0x2e62u32, 0x2e63u32, 0x1ceddu32, 0x1cedeu32, 0x1cedfu32, 0x1db03u32, 0x1db04u32,
        0x1db05u32, 0x1db06u32, 0x1db07u32, 0x1db08u32, 0x1db09u32, 0x1db10u32, 0x1db17u32,
        0x1db18u32, 0x1db1bu32,
    ] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Bidi_Mirrored", "Bidi_M"] {
            for flags in ["u", "ui", "v", "vi"] {
                check(&format!(
                    r"let p=new RegExp('^\\p{{{name}}}$','{flags}'),n=new RegExp('^\\P{{{name}}}$','{flags}');p.test({text:?})&&!n.test({text:?})"
                ));
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 128);
}
