//! Soft_Dotted and its exact SD alias over complete Unicode characters.
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
fn unicode_soft_dotted_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Soft_Dotted}",
        r"\P{SD}",
        r"[^\p{SD}]",
        r"[\p{SD}K]",
        r"[\P{Soft_Dotted}K]",
        r"^(?<x>[\p{SD}K])$",
        r"(?<x>\P{Soft_Dotted})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{49}\u{4a}\u{68}\u{69}\u{6a}\u{6b}\u{12e}\u{12f}\u{130}\u{197}\u{248}\u{249}\u{24a}\u{267}\u{268}\u{269}\u{29c}\u{29d}\u{29e}\u{2b1}\u{2b2}\u{2b3}\u{37f}\u{3f2}\u{3f3}\u{3f4}\u{406}\u{408}\u{455}\u{456}\u{457}\u{458}\u{459}\u{1d61}\u{1d62}\u{1d63}\u{1d95}\u{1d96}\u{1d97}\u{1da3}\u{1da4}\u{1da5}\u{1da7}\u{1da8}\u{1da9}\u{1e2c}\u{1e2d}\u{1e2e}\u{1eca}\u{1ecb}\u{1ecc}\u{2070}\u{2071}\u{2072}\u{2147}\u{2148}\u{2149}\u{214a}\u{2c7b}\u{2c7c}\u{2c7d}\u{a7b2}\u{1d421}\u{1d422}\u{1d423}\u{1d424}\u{1d455}\u{1d456}\u{1d457}\u{1d458}\u{1d489}\u{1d48a}\u{1d48b}\u{1d48c}\u{1d4bd}\u{1d4be}\u{1d4bf}\u{1d4c0}\u{1d4f1}\u{1d4f2}\u{1d4f3}\u{1d4f4}\u{1d525}\u{1d526}\u{1d527}\u{1d528}\u{1d559}\u{1d55a}\u{1d55b}\u{1d55c}\u{1d58d}\u{1d58e}\u{1d58f}\u{1d590}\u{1d5c1}\u{1d5c2}\u{1d5c3}\u{1d5c4}\u{1d5f5}\u{1d5f6}\u{1d5f7}\u{1d5f8}\u{1d629}\u{1d62a}\u{1d62b}\u{1d62c}\u{1d65d}\u{1d65e}\u{1d65f}\u{1d660}\u{1d691}\u{1d692}\u{1d693}\u{1d694}\u{1df19}\u{1df1a}\u{1df1b}\u{1df72}\u{1dfd8}\u{1dfda}\u{1e04b}\u{1e04c}\u{1e04d}\u{1e04e}\u{1e067}\u{1e068}\u{1e069}\u{feff}ſK\n",
                "X\r\n\u{69}\u{2028}\u{6a}\u{2029}",
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
fn unicode_soft_dotted_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let p=/^\p{{SD}}$/i{mode},n=/^\P{{SD}}$/i{mode};'\u{{49}}\u{{4a}}\u{{69}}\u{{6a}}\u{{12e}}\u{{12f}}\u{{197}}\u{{248}}\u{{249}}\u{{268}}\u{{29d}}\u{{37f}}\u{{3f3}}\u{{406}}\u{{408}}\u{{456}}\u{{458}}\u{{1e2c}}\u{{1e2d}}\u{{1eca}}\u{{1ecb}}\u{{a7b2}}'.split('').every(c=>p.test(c)&&n.test(c)===('{mode}'==='u'))"
        ));
        check(&format!(
            r"let p=/[\p{{SD}}K]/i{mode},n=/[\P{{SD}}K]/i{mode},x=/[^\P{{SD}}K]/i{mode};p.test('K')&&n.test('K')&&!x.test('K')&&'\u{{49}}\u{{4a}}\u{{69}}\u{{6a}}\u{{12e}}\u{{12f}}\u{{197}}\u{{248}}\u{{249}}\u{{268}}\u{{29d}}\u{{37f}}\u{{3f3}}\u{{406}}\u{{408}}\u{{456}}\u{{458}}\u{{1e2c}}\u{{1e2d}}\u{{1eca}}\u{{1ecb}}\u{{a7b2}}'.split('').every(c=>p.test(c)&&n.test(c)===('{mode}'==='u')&&x.test(c)===('{mode}'==='v'))"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Soft_Dotted}}/{mode},b=/\p{{SD}}/i{mode},n=/\P{{SD}}/i{mode};a.test('\u0069')&&a.test('\u006A')&&b.test('\u0069')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&n.test('\u0069')===('{mode}'==='u')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Soft_Dotted}}/{mode},s=/\s/{mode};p.test('\u0069')&&p.test('\u006A')&&!s.test('\u0069')&&!s.test('\u006A')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{SD}}K]/i{mode},b=/[^\P{{Soft_Dotted}}K]/i{mode},c=/[\p{{SD}}\P{{Soft_Dotted}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u0069')&&!a.test('ſ')&&b.test('\u006A')===('{mode}'==='v')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{SD}}$/{mode},b=/^\P{{SD}}$/{mode};a.test('\u0069')&&!a.test('\u006A\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_soft_dotted_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{SD}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uD835\uDC22');m.index===0&&m[0]==='\uD835\uDC22'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{SD}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{SD}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u0069');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{SD}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u0069');m.index===2&&m[0]==='\u0069'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{SD}}/{mode}y;r.lastIndex=1;r.exec('😀\u0069')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_soft_dotted_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u006A😀\u0069G'.matchAll(/(?<x>\P{{SD}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u006A\u0069😀X'.replace(/(?<x>\p{{Soft_Dotted}})/{mode}g,'<$1:$<x>>')==='<\u006A:\u006A><\u0069:\u0069>😀X'"
        ));
        check(&format!(
            r"'😀\u006A\u0069G'.split(/(\p{{SD}})/{mode}).join('|')==='😀|\u006A||\u0069|G'&&'😀\u0069'.search(/\p{{SD}}/{mode})===2&&'\u006A\u0069😀'.match(/\p{{SD}}/{mode}g).join('')==='\u006A\u0069'"
        ));
        check(&format!(
            r"let t='';let x='\u0069\u006A😀\u0069'.replace(/(\p{{SD}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_soft_dotted_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Soft_Dotted})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Soft_Dotted})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u0069').groups.x==='\u0069'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{SD}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{SD}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u0069')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{SD}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u0069')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_soft_dotted_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{SD}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u0069'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u0069'&&m[3000]==='\\u0069'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_soft_dotted_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{SD}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    check(r"/[\p{SD}&&A]/v.exec('A')===null");
    check(r"/[\p{SD}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x69),m=/[\p{SD}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [r"/\p{SD}+b/u.exec('AA')", r"/\p{SD}A/v.exec('AA')"] {
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
fn unicode_soft_dotted_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\b(?<x>\p{{SD}})+\b/d{mode}g,m=r.exec('😀\u0069\u006A\u006A\uFEFF');m.index===2&&m[0]==='\u0069\u006A\u006A'&&m.groups.x==='\u006A'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_soft_dotted_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [0x1df6eu32, 0x1df6fu32, 0x1df70u32, 0x1df71u32, 0x1dfd9u32] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Soft_Dotted", "SD"] {
            for flags in ["u", "ui", "v", "vi"] {
                check(&format!(
                    r"let p=new RegExp('^\\p{{{name}}}$','{flags}'),n=new RegExp('^\\P{{{name}}}$','{flags}');p.test({text:?})===({point}!==122734||'{flags}'.includes('i'))&&n.test({text:?})===({point}===122734?'{flags}'!=='vi':('{flags}'==='ui'&&{point}===122735))"
                ));
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 40);
}
