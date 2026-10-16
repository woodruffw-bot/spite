//! Emoji and its exact Emoji alias over complete Unicode characters.
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
fn unicode_emoji_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Emoji}",
        r"\P{Emoji}",
        r"[^\p{Emoji}]",
        r"[\p{Emoji}K]",
        r"[\P{Emoji}K]",
        r"^(?<x>[\p{Emoji}K])$",
        r"(?<x>\P{Emoji})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "\u{e0001}\u{22}\u{23}\u{24}\u{29}\u{2a}\u{2b}\u{2f}\u{30}\u{39}\u{3a}\u{a8}\u{a9}\u{aa}\u{ad}\u{ae}\u{af}\u{203b}\u{203c}\u{203d}\u{2048}\u{2049}\u{204a}\u{2121}\u{2122}\u{2123}\u{2138}\u{2139}\u{213a}\u{2193}\u{2194}\u{2199}\u{219a}\u{21a8}\u{21a9}\u{21aa}\u{21ab}\u{2319}\u{231a}\u{231b}\u{231c}\u{2327}\u{2328}\u{2329}\u{23ce}\u{23cf}\u{23d0}\u{23e8}\u{23e9}\u{23f3}\u{23f4}\u{23f7}\u{23f8}\u{23fa}\u{23fb}\u{24c1}\u{24c2}\u{24c3}\u{24dc}\u{25a9}\u{25aa}\u{25ab}\u{25ac}\u{25b5}\u{25b6}\u{25b7}\u{25bf}\u{25c0}\u{25c1}\u{25fa}\u{25fb}\u{25fe}\u{25ff}\u{2600}\u{2604}\u{2605}\u{260d}\u{260e}\u{260f}\u{2610}\u{2611}\u{2612}\u{2613}\u{2614}\u{2615}\u{2616}\u{2617}\u{2618}\u{2619}\u{261c}\u{261d}\u{261e}\u{261f}\u{2620}\u{2621}\u{2622}\u{2623}\u{2624}\u{2625}\u{2626}\u{2627}\u{2629}\u{262a}\u{262b}\u{262d}\u{262e}\u{262f}\u{2630}\u{2637}\u{2638}\u{263a}\u{263b}\u{263f}\u{2640}\u{2641}\u{2642}\u{2643}\u{2647}\u{2648}\u{2653}\u{2654}\u{265e}\u{265f}\u{2660}\u{2661}\u{2662}\u{2663}\u{2664}\u{2665}\u{2666}\u{2667}\u{2668}\u{2669}\u{267a}\u{267b}\u{267c}\u{267d}\u{267e}\u{267f}\u{2680}\u{2691}\u{2692}\u{2697}\u{2698}\u{2699}\u{269a}\u{269b}\u{269c}\u{269d}\u{269f}\u{26a0}\u{26a1}\u{26a2}\u{26a6}\u{26a7}\u{26a8}\u{26a9}\u{26aa}\u{26ab}\u{26ac}\u{26af}\u{26b0}\u{26b1}\u{26b2}\u{26bc}\u{26bd}\u{26be}\u{26bf}\u{26c3}\u{26c4}\u{26c5}\u{26c6}\u{26c7}\u{26c8}\u{26c9}\u{26cd}\u{26ce}\u{26cf}\u{26d0}\u{26d1}\u{26d2}\u{26d3}\u{26d4}\u{26d5}\u{26e8}\u{26e9}\u{26ea}\u{26eb}\u{26ef}\u{26f0}\u{26f5}\u{26f6}\u{26f7}\u{26fa}\u{26fb}\u{26fc}\u{26fd}\u{26fe}\u{2701}\u{2702}\u{2703}\u{2704}\u{2705}\u{2706}\u{2707}\u{2708}\u{270d}\u{270e}\u{270f}\u{2710}\u{2711}\u{2712}\u{2713}\u{2714}\u{2715}\u{2716}\u{2717}\u{271c}\u{271d}\u{271e}\u{2720}\u{2721}\u{2722}\u{2727}\u{2728}\u{2729}\u{2732}\u{2733}\u{2734}\u{2735}\u{2743}\u{2744}\u{2745}\u{2746}\u{2747}\u{2748}\u{274b}\u{274c}\u{274d}\u{274e}\u{274f}\u{2752}\u{2753}\u{2755}\u{2756}\u{2757}\u{2758}\u{2762}\u{2763}\u{2764}\u{2765}\u{2794}\u{2795}\u{2797}\u{2798}\u{27a0}\u{27a1}\u{27a2}\u{27af}\u{27b0}\u{27b1}\u{27be}\u{27bf}\u{27c0}\u{2933}\u{2934}\u{2935}\u{2936}\u{2b04}\u{2b05}\u{2b07}\u{2b08}\u{2b1a}\u{2b1b}\u{2b1c}\u{2b1d}\u{2b4f}\u{2b50}\u{2b51}\u{2b54}\u{2b55}\u{2b56}\u{302f}\u{3030}\u{3031}\u{303c}\u{303d}\u{303e}\u{3296}\u{3297}\u{3298}\u{3299}\u{329a}\u{1f003}\u{1f004}\u{1f005}\u{1f0ce}\u{1f0cf}\u{1f0d0}\u{1f16f}\u{1f170}\u{1f171}\u{1f172}\u{1f17d}\u{1f17e}\u{1f17f}\u{1f180}\u{1f18d}\u{1f18e}\u{1f18f}\u{1f190}\u{1f191}\u{1f19a}\u{1f19b}\u{1f1e5}\u{1f1e6}\u{1f1ff}\u{1f200}\u{1f201}\u{1f202}\u{1f203}\u{1f219}\u{1f21a}\u{1f21b}\u{1f22e}\u{1f22f}\u{1f230}\u{1f231}\u{1f232}\u{1f23a}\u{1f23b}\u{1f24f}\u{1f250}\u{1f251}\u{1f252}\u{1f2ff}\u{1f300}\u{1f321}\u{1f322}\u{1f323}\u{1f324}\u{1f393}\u{1f394}\u{1f395}\u{1f396}\u{1f397}\u{1f398}\u{1f399}\u{1f39b}\u{1f39c}\u{1f39d}\u{1f39e}\u{1f3f0}\u{1f3f1}\u{1f3f2}\u{1f3f3}\u{1f3f5}\u{1f3f6}\u{1f3f7}\u{1f4fd}\u{1f4fe}\u{1f4ff}\u{1f53d}\u{1f53e}\u{1f548}\u{1f549}\u{1f54e}\u{1f54f}\u{1f550}\u{1f567}\u{1f568}\u{1f56e}\u{1f56f}\u{1f570}\u{1f571}\u{1f572}\u{1f573}\u{1f57a}\u{1f57b}\u{1f586}\u{1f587}\u{1f588}\u{1f589}\u{1f58a}\u{1f58d}\u{1f58e}\u{1f58f}\u{1f590}\u{1f591}\u{1f594}\u{1f595}\u{1f596}\u{1f597}\u{1f5a3}\u{1f5a4}\u{1f5a5}\u{1f5a6}\u{1f5a7}\u{1f5a8}\u{1f5a9}\u{1f5b0}\u{1f5b1}\u{1f5b2}\u{1f5b3}\u{1f5bb}\u{1f5bc}\u{1f5bd}\u{1f5c1}\u{1f5c2}\u{1f5c4}\u{1f5c5}\u{1f5d0}\u{1f5d1}\u{1f5d3}\u{1f5d4}\u{1f5db}\u{1f5dc}\u{1f5de}\u{1f5df}\u{1f5e0}\u{1f5e1}\u{1f5e2}\u{1f5e3}\u{1f5e4}\u{1f5e7}\u{1f5e8}\u{1f5e9}\u{1f5ee}\u{1f5ef}\u{1f5f0}\u{1f5f2}\u{1f5f3}\u{1f5f4}\u{1f5f9}\u{1f5fa}\u{1f64f}\u{1f650}\u{1f67f}\u{1f680}\u{1f6c5}\u{1f6c6}\u{1f6ca}\u{1f6cb}\u{1f6d2}\u{1f6d3}\u{1f6d4}\u{1f6d5}\u{1f6da}\u{1f6db}\u{1f6dc}\u{1f6e5}\u{1f6e6}\u{1f6e8}\u{1f6e9}\u{1f6ea}\u{1f6eb}\u{1f6ec}\u{1f6ed}\u{1f6ef}\u{1f6f0}\u{1f6f1}\u{1f6f2}\u{1f6f3}\u{1f6fc}\u{1f6fd}\u{1f7df}\u{1f7e0}\u{1f7eb}\u{1f7ec}\u{1f7ef}\u{1f7f0}\u{1f7f1}\u{1f90b}\u{1f90c}\u{1f93a}\u{1f93b}\u{1f93c}\u{1f945}\u{1f946}\u{1f947}\u{1f9ff}\u{1fa00}\u{1fa6f}\u{1fa70}\u{1fa7c}\u{1fa7d}\u{1fa7f}\u{1fa80}\u{1fac6}\u{1fac7}\u{1fac8}\u{1fac9}\u{1facb}\u{1fade}\u{1fadf}\u{1faec}\u{1faee}\u{1faef}\u{1fafb}\u{feff}ſK\n",
                "X\r\n\u{a9}\u{2028}\u{ae}\u{2029}",
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
fn unicode_emoji_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let p=/^\p{{Emoji}}$/i{mode},n=/^\P{{Emoji}}$/i{mode};[...'\u24C2\u24DC'].every(c=>p.test(c)&&n.test(c)===('{mode}'==='u'))"
        ));
        check(&format!(
            r"let p=/[\p{{Emoji}}K]/i{mode},n=/[\P{{Emoji}}K]/i{mode},x=/[^\P{{Emoji}}K]/i{mode};p.test('K')&&n.test('K')&&!x.test('K')&&[...'\u24C2\u24DC'].every(c=>p.test(c)&&n.test(c)===('{mode}'==='u')&&x.test(c)===('{mode}'==='v'))"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Emoji}}/{mode},b=/\p{{Emoji}}/i{mode},n=/\P{{Emoji}}/i{mode};a.test('\u00A9')&&a.test('\u00AE')&&b.test('\u00A9')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u00A9')&&n.test('\uDB40\uDC01')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Emoji}}/{mode},s=/\s/{mode};p.test('\u00A9')&&p.test('\u00AE')&&!s.test('\u00A9')&&!s.test('\u00AE')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Emoji}}K]/i{mode},b=/[^\P{{Emoji}}K]/i{mode},c=/[\p{{Emoji}}\P{{Emoji}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u00A9')&&!a.test('ſ')&&b.test('\u00AE')&&!b.test('K')&&c.test('\uDB40\uDC01')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Emoji}}$/{mode},b=/^\P{{Emoji}}$/{mode};a.test('\u00A9')&&!a.test('\u00AE\n')&&!a.test('')&&!a.test('\n')&&b.test('\uDB40\uDC01')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_emoji_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{Emoji}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uD83D\uDE00');m.index===0&&m[0]==='\uD83D\uDE00'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Emoji}}K]))$/dim{mode}g,m=r.exec('\u200D\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Emoji}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u00A9');m.index===0&&m[1]==='\uDB40\uDC01'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Emoji}}/d{mode}g;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u00A9');m.index===2&&m[0]==='\u00A9'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Emoji}}/{mode}y;r.lastIndex=1;r.exec('\uDB40\uDC01\u00A9')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_emoji_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u00AE\uDB40\uDC01\u00A9\u200D'.matchAll(/(?<x>\P{{Emoji}})/d{mode}g)];a.length===2&&a[0][1]==='\uDB40\uDC01'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='\u200D'"
        ));
        check(&format!(
            r"'\u00AE\u00A9\uDB40\uDC01\u200D'.replace(/(?<x>\p{{Emoji}})/{mode}g,'<$1:$<x>>')==='<\u00AE:\u00AE><\u00A9:\u00A9>\uDB40\uDC01\u200D'"
        ));
        check(&format!(
            r"'\uDB40\uDC01\u00AE\u00A9\u200D'.split(/(\p{{Emoji}})/{mode}).join('|')==='\uDB40\uDC01|\u00AE||\u00A9|\u200D'&&'\uDB40\uDC01\u00A9'.search(/\p{{Emoji}}/{mode})===2&&'\u00AE\u00A9\uDB40\uDC01'.match(/\p{{Emoji}}/{mode}g).join('')==='\u00AE\u00A9'"
        ));
        check(&format!(
            r"let t='';let x='\u00A9\u00AE\uDB40\uDC01\u00A9'.replace(/(\p{{Emoji}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__\uDB40\uDC01_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_emoji_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Emoji})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Emoji})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u00A9').groups.x==='\u00A9'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Emoji}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '\uDB40\uDC01'}});t==='si'&&m[0]==='\uDB40\uDC01'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Emoji}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u00A9')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Emoji}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\uDB40\uDC01\u00A9')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_emoji_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{Emoji}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='\\uDB40\\uDC01'.repeat(100000)+'\\n\\u00A9'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u00A9'&&m[3000]==='\\u00A9'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_emoji_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Emoji}/ug,marker=0,text='\\uDB40\\uDC01'.repeat(10000);r.lastIndex=1")
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
    check(r"/[\p{Emoji}&&A]/v.exec('A')===null");
    check(r"/[\p{Emoji}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0xa9),m=/[\p{Emoji}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [r"/\p{Emoji}+b/u.exec('AA')", r"/\p{Emoji}A/v.exec('AA')"] {
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
fn unicode_emoji_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{Emoji}})+\B/d{mode}g,m=r.exec('\uDB40\uDC01\u00A9\u00AE\u00AE\uFEFF');m.index===2&&m[0]==='\u00A9\u00AE\u00AE'&&m.groups.x==='\u00AE'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_emoji_property_unicode18_membership_changes() {
    check(
        r"let p=new RegExp('^\\p{Emoji}$','u'),n=new RegExp('^\\P{Emoji}$','u');p.test('\uD83D\uDED9')===true&&n.test('\uD83D\uDED9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','ui'),n=new RegExp('^\\P{Emoji}$','ui');p.test('\uD83D\uDED9')===true&&n.test('\uD83D\uDED9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','v'),n=new RegExp('^\\P{Emoji}$','v');p.test('\uD83D\uDED9')===true&&n.test('\uD83D\uDED9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','vi'),n=new RegExp('^\\P{Emoji}$','vi');p.test('\uD83D\uDED9')===true&&n.test('\uD83D\uDED9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','u'),n=new RegExp('^\\P{Emoji}$','u');p.test('\uD83E\uDE8B')===true&&n.test('\uD83E\uDE8B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','ui'),n=new RegExp('^\\P{Emoji}$','ui');p.test('\uD83E\uDE8B')===true&&n.test('\uD83E\uDE8B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','v'),n=new RegExp('^\\P{Emoji}$','v');p.test('\uD83E\uDE8B')===true&&n.test('\uD83E\uDE8B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','vi'),n=new RegExp('^\\P{Emoji}$','vi');p.test('\uD83E\uDE8B')===true&&n.test('\uD83E\uDE8B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','u'),n=new RegExp('^\\P{Emoji}$','u');p.test('\uD83E\uDE8C')===true&&n.test('\uD83E\uDE8C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','ui'),n=new RegExp('^\\P{Emoji}$','ui');p.test('\uD83E\uDE8C')===true&&n.test('\uD83E\uDE8C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','v'),n=new RegExp('^\\P{Emoji}$','v');p.test('\uD83E\uDE8C')===true&&n.test('\uD83E\uDE8C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','vi'),n=new RegExp('^\\P{Emoji}$','vi');p.test('\uD83E\uDE8C')===true&&n.test('\uD83E\uDE8C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','u'),n=new RegExp('^\\P{Emoji}$','u');p.test('\uD83E\uDE8D')===true&&n.test('\uD83E\uDE8D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','ui'),n=new RegExp('^\\P{Emoji}$','ui');p.test('\uD83E\uDE8D')===true&&n.test('\uD83E\uDE8D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','v'),n=new RegExp('^\\P{Emoji}$','v');p.test('\uD83E\uDE8D')===true&&n.test('\uD83E\uDE8D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','vi'),n=new RegExp('^\\P{Emoji}$','vi');p.test('\uD83E\uDE8D')===true&&n.test('\uD83E\uDE8D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','u'),n=new RegExp('^\\P{Emoji}$','u');p.test('\uD83E\uDECC')===true&&n.test('\uD83E\uDECC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','ui'),n=new RegExp('^\\P{Emoji}$','ui');p.test('\uD83E\uDECC')===true&&n.test('\uD83E\uDECC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','v'),n=new RegExp('^\\P{Emoji}$','v');p.test('\uD83E\uDECC')===true&&n.test('\uD83E\uDECC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','vi'),n=new RegExp('^\\P{Emoji}$','vi');p.test('\uD83E\uDECC')===true&&n.test('\uD83E\uDECC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','u'),n=new RegExp('^\\P{Emoji}$','u');p.test('\uD83E\uDEDD')===true&&n.test('\uD83E\uDEDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','ui'),n=new RegExp('^\\P{Emoji}$','ui');p.test('\uD83E\uDEDD')===true&&n.test('\uD83E\uDEDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','v'),n=new RegExp('^\\P{Emoji}$','v');p.test('\uD83E\uDEDD')===true&&n.test('\uD83E\uDEDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','vi'),n=new RegExp('^\\P{Emoji}$','vi');p.test('\uD83E\uDEDD')===true&&n.test('\uD83E\uDEDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','u'),n=new RegExp('^\\P{Emoji}$','u');p.test('\uD83E\uDEEB')===true&&n.test('\uD83E\uDEEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','ui'),n=new RegExp('^\\P{Emoji}$','ui');p.test('\uD83E\uDEEB')===true&&n.test('\uD83E\uDEEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','v'),n=new RegExp('^\\P{Emoji}$','v');p.test('\uD83E\uDEEB')===true&&n.test('\uD83E\uDEEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','vi'),n=new RegExp('^\\P{Emoji}$','vi');p.test('\uD83E\uDEEB')===true&&n.test('\uD83E\uDEEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','u'),n=new RegExp('^\\P{Emoji}$','u');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','ui'),n=new RegExp('^\\P{Emoji}$','ui');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','v'),n=new RegExp('^\\P{Emoji}$','v');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','vi'),n=new RegExp('^\\P{Emoji}$','vi');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','u'),n=new RegExp('^\\P{Emoji}$','u');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','ui'),n=new RegExp('^\\P{Emoji}$','ui');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','v'),n=new RegExp('^\\P{Emoji}$','v');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji}$','vi'),n=new RegExp('^\\P{Emoji}$','vi');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
}
