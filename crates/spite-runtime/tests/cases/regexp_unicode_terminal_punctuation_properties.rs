//! Terminal_Punctuation and its exact Term alias over complete Unicode characters.
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
fn unicode_terminal_punctuation_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Terminal_Punctuation}",
        r"\P{Term}",
        r"[^\p{Term}]",
        r"[\p{Term}K]",
        r"[\P{Terminal_Punctuation}K]",
        r"^(?<x>[\p{Term}K])$",
        r"(?<x>\P{Terminal_Punctuation})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{20}\u{21}\u{22}\u{2b}\u{2c}\u{2d}\u{2e}\u{2f}\u{39}\u{3a}\u{3b}\u{3c}\u{3e}\u{3f}\u{40}\u{37d}\u{37e}\u{37f}\u{386}\u{387}\u{388}\u{588}\u{589}\u{58a}\u{5c2}\u{5c3}\u{5c4}\u{60b}\u{60c}\u{60d}\u{61a}\u{61b}\u{61c}\u{61d}\u{61f}\u{620}\u{6d3}\u{6d4}\u{6d5}\u{6ff}\u{700}\u{70a}\u{70b}\u{70c}\u{70d}\u{7f7}\u{7f8}\u{7f9}\u{7fa}\u{82f}\u{830}\u{835}\u{836}\u{837}\u{83e}\u{83f}\u{85d}\u{85e}\u{85f}\u{963}\u{964}\u{965}\u{966}\u{e59}\u{e5a}\u{e5b}\u{e5c}\u{f07}\u{f08}\u{f09}\u{f0c}\u{f0d}\u{f12}\u{f13}\u{1049}\u{104a}\u{104b}\u{104c}\u{1360}\u{1361}\u{1368}\u{1369}\u{166d}\u{166e}\u{166f}\u{16ea}\u{16eb}\u{16ed}\u{16ee}\u{1734}\u{1735}\u{1736}\u{1737}\u{17d3}\u{17d4}\u{17d6}\u{17d7}\u{17d9}\u{17da}\u{17db}\u{1801}\u{1802}\u{1805}\u{1806}\u{1807}\u{1808}\u{1809}\u{180a}\u{1943}\u{1944}\u{1945}\u{1946}\u{1aa7}\u{1aa8}\u{1aab}\u{1aac}\u{1b4d}\u{1b4e}\u{1b4f}\u{1b50}\u{1b59}\u{1b5a}\u{1b5b}\u{1b5c}\u{1b5d}\u{1b5f}\u{1b60}\u{1b7c}\u{1b7d}\u{1b7f}\u{1b80}\u{1c3a}\u{1c3b}\u{1c3f}\u{1c40}\u{1c7d}\u{1c7e}\u{1c7f}\u{1c80}\u{2023}\u{2024}\u{2025}\u{203b}\u{203c}\u{203d}\u{203e}\u{2046}\u{2047}\u{2049}\u{204a}\u{2cf8}\u{2cf9}\u{2cfb}\u{2cfc}\u{2e2d}\u{2e2e}\u{2e2f}\u{2e3b}\u{2e3c}\u{2e3d}\u{2e40}\u{2e41}\u{2e42}\u{2e4b}\u{2e4c}\u{2e4d}\u{2e4e}\u{2e4f}\u{2e50}\u{2e52}\u{2e53}\u{2e54}\u{2e55}\u{2e5f}\u{2e62}\u{3000}\u{3001}\u{3002}\u{3003}\u{a4fd}\u{a4fe}\u{a4ff}\u{a500}\u{a60c}\u{a60d}\u{a60f}\u{a610}\u{a6f2}\u{a6f3}\u{a6f7}\u{a6f8}\u{a875}\u{a876}\u{a877}\u{a878}\u{a8cd}\u{a8ce}\u{a8cf}\u{a8d0}\u{a92e}\u{a92f}\u{a930}\u{a9c6}\u{a9c7}\u{a9c9}\u{a9ca}\u{aa5c}\u{aa5d}\u{aa5f}\u{aa60}\u{aade}\u{aadf}\u{aae0}\u{aaef}\u{aaf0}\u{aaf1}\u{aaf2}\u{abea}\u{abeb}\u{abec}\u{fe11}\u{fe12}\u{fe13}\u{fe14}\u{fe15}\u{fe16}\u{fe17}\u{fe4f}\u{fe50}\u{fe52}\u{fe53}\u{fe54}\u{fe57}\u{fe58}\u{ff00}\u{ff01}\u{ff02}\u{ff0b}\u{ff0c}\u{ff0d}\u{ff0e}\u{ff0f}\u{ff19}\u{ff1a}\u{ff1b}\u{ff1c}\u{ff1e}\u{ff1f}\u{ff20}\u{ff60}\u{ff61}\u{ff62}\u{ff63}\u{ff64}\u{ff65}\u{1039e}\u{1039f}\u{103a0}\u{103cf}\u{103d0}\u{103d1}\u{10856}\u{10857}\u{10858}\u{1091e}\u{1091f}\u{10920}\u{10a55}\u{10a56}\u{10a57}\u{10a58}\u{10aef}\u{10af0}\u{10af5}\u{10af6}\u{10b39}\u{10b3a}\u{10b3f}\u{10b40}\u{10b98}\u{10b99}\u{10b9c}\u{10b9d}\u{10f54}\u{10f55}\u{10f59}\u{10f5a}\u{10f85}\u{10f86}\u{10f89}\u{10f8a}\u{11046}\u{11047}\u{1104d}\u{1104e}\u{110bd}\u{110be}\u{110c1}\u{110c2}\u{11140}\u{11141}\u{11143}\u{11144}\u{111c4}\u{111c5}\u{111c6}\u{111c7}\u{111cc}\u{111cd}\u{111ce}\u{111dd}\u{111de}\u{111df}\u{111e0}\u{11237}\u{11238}\u{1123c}\u{1123d}\u{112a8}\u{112a9}\u{112aa}\u{113d3}\u{113d4}\u{113d5}\u{113d6}\u{1144a}\u{1144b}\u{1144d}\u{1144e}\u{11459}\u{1145a}\u{1145b}\u{1145c}\u{115c1}\u{115c2}\u{115c5}\u{115c6}\u{115c8}\u{115c9}\u{115d7}\u{115d8}\u{11640}\u{11641}\u{11642}\u{11643}\u{1173b}\u{1173c}\u{1173e}\u{1173f}\u{11943}\u{11944}\u{11945}\u{11946}\u{11947}\u{11a41}\u{11a42}\u{11a43}\u{11a44}\u{11a9a}\u{11a9b}\u{11a9c}\u{11a9d}\u{11aa0}\u{11aa1}\u{11aa2}\u{11aa3}\u{11c40}\u{11c41}\u{11c43}\u{11c44}\u{11c70}\u{11c71}\u{11c72}\u{11ef6}\u{11ef7}\u{11ef8}\u{11ef9}\u{11f42}\u{11f43}\u{11f44}\u{11f45}\u{1246f}\u{12470}\u{12474}\u{12475}\u{16a6d}\u{16a6e}\u{16a6f}\u{16a70}\u{16af4}\u{16af5}\u{16af6}\u{16b36}\u{16b37}\u{16b39}\u{16b3a}\u{16b43}\u{16b44}\u{16b45}\u{16d6d}\u{16d6e}\u{16d6f}\u{16d70}\u{16e96}\u{16e97}\u{16e98}\u{16e99}\u{1bc9e}\u{1bc9f}\u{1bca0}\u{1da86}\u{1da87}\u{1da8a}\u{1da8b}\u{feff}ſK\n",
                "X\r\n\u{21}\u{2028}\u{3002}\u{2029}",
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
fn unicode_terminal_punctuation_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Terminal_Punctuation}}/{mode},b=/\p{{Term}}/i{mode},n=/\P{{Term}}/i{mode};a.test('\u0021')&&a.test('\u003F')&&b.test('\u0021')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u0021')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Terminal_Punctuation}}/{mode},s=/\s/{mode};p.test('\u0021')&&p.test('\u003F')&&!s.test('\u0021')&&!s.test('\u003F')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Term}}K]/i{mode},b=/[^\P{{Terminal_Punctuation}}K]/i{mode},c=/[\p{{Term}}\P{{Terminal_Punctuation}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u0021')&&!a.test('ſ')&&b.test('\u003F')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Term}}$/{mode},b=/^\P{{Term}}$/{mode};a.test('\u0021')&&!a.test('\u003F\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_terminal_punctuation_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Term}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Term}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u0021');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Term}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u0021');m.index===2&&m[0]==='\u0021'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Term}}/{mode}y;r.lastIndex=1;r.exec('😀\u0021')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_terminal_punctuation_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u003F😀\u0021G'.matchAll(/(?<x>\P{{Term}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u003F\u0021😀X'.replace(/(?<x>\p{{Terminal_Punctuation}})/{mode}g,'<$1:$<x>>')==='<\u003F:\u003F><\u0021:\u0021>😀X'"
        ));
        check(&format!(
            r"'😀\u003F\u0021G'.split(/(\p{{Term}})/{mode}).join('|')==='😀|\u003F||\u0021|G'&&'😀\u0021'.search(/\p{{Term}}/{mode})===2&&'\u003F\u0021😀'.match(/\p{{Term}}/{mode}g).join('')==='\u003F\u0021'"
        ));
        check(&format!(
            r"let t='';let x='\u0021\u003F😀\u0021'.replace(/(\p{{Term}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_terminal_punctuation_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Terminal_Punctuation})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Terminal_Punctuation})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u0021').groups.x==='\u0021'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Term}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Term}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u0021')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Term}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u0021')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_terminal_punctuation_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Term}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    check(r"/[\p{Term}&&A]/v.exec('A')===null");
    check(r"/[\p{Term}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x21),m=/[\p{Term}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [r"/\p{Term}+b/u.exec('AA')", r"/\p{Term}A/v.exec('AA')"] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}

#[test]
fn unicode_terminal_punctuation_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{Term}})+\B/d{mode}g,m=r.exec('😀\u0021\u003F\u3002\uFEFF');m.index===2&&m[0]==='\u0021\u003F\u3002'&&m.groups.x==='\u3002'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_terminal_punctuation_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [0x2e60u32, 0x2e61] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Terminal_Punctuation", "Term"] {
            for flags in ["u", "ui", "v", "vi"] {
                check(&format!(
                    r"let p=new RegExp('^\\p{{{name}}}$','{flags}'),n=new RegExp('^\\P{{{name}}}$','{flags}');p.test({text:?})&&!n.test({text:?})"
                ));
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 16);
}
