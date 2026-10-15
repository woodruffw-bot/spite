//! Emoji_Presentation and its exact EPres alias over complete Unicode characters.
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
fn unicode_emoji_presentation_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Emoji_Presentation}",
        r"\P{EPres}",
        r"[^\p{EPres}]",
        r"[\p{EPres}K]",
        r"[\P{Emoji_Presentation}K]",
        r"^(?<x>[\p{EPres}K])$",
        r"(?<x>\P{Emoji_Presentation})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "\u{e0001}\u{2319}\u{231a}\u{231b}\u{231c}\u{23e8}\u{23e9}\u{23ec}\u{23ed}\u{23ef}\u{23f0}\u{23f1}\u{23f2}\u{23f3}\u{23f4}\u{25fc}\u{25fd}\u{25fe}\u{25ff}\u{2613}\u{2614}\u{2615}\u{2616}\u{2647}\u{2648}\u{2653}\u{2654}\u{267e}\u{267f}\u{2680}\u{2692}\u{2693}\u{2694}\u{26a0}\u{26a1}\u{26a2}\u{26a9}\u{26aa}\u{26ab}\u{26ac}\u{26bc}\u{26bd}\u{26be}\u{26bf}\u{26c3}\u{26c4}\u{26c5}\u{26c6}\u{26cd}\u{26ce}\u{26cf}\u{26d3}\u{26d4}\u{26d5}\u{26e9}\u{26ea}\u{26eb}\u{26f1}\u{26f2}\u{26f3}\u{26f4}\u{26f5}\u{26f6}\u{26f9}\u{26fa}\u{26fb}\u{26fc}\u{26fd}\u{26fe}\u{2704}\u{2705}\u{2706}\u{2709}\u{270a}\u{270b}\u{270c}\u{2727}\u{2728}\u{2729}\u{274b}\u{274c}\u{274d}\u{274e}\u{274f}\u{2752}\u{2753}\u{2755}\u{2756}\u{2757}\u{2758}\u{2794}\u{2795}\u{2797}\u{2798}\u{27af}\u{27b0}\u{27b1}\u{27be}\u{27bf}\u{27c0}\u{2b1a}\u{2b1b}\u{2b1c}\u{2b1d}\u{2b4f}\u{2b50}\u{2b51}\u{2b54}\u{2b55}\u{2b56}\u{1f003}\u{1f004}\u{1f005}\u{1f0ce}\u{1f0cf}\u{1f0d0}\u{1f18d}\u{1f18e}\u{1f18f}\u{1f190}\u{1f191}\u{1f19a}\u{1f19b}\u{1f1e5}\u{1f1e6}\u{1f1ff}\u{1f200}\u{1f201}\u{1f202}\u{1f219}\u{1f21a}\u{1f21b}\u{1f22e}\u{1f22f}\u{1f230}\u{1f231}\u{1f232}\u{1f236}\u{1f237}\u{1f238}\u{1f23a}\u{1f23b}\u{1f24f}\u{1f250}\u{1f251}\u{1f252}\u{1f2ff}\u{1f300}\u{1f320}\u{1f321}\u{1f32c}\u{1f32d}\u{1f335}\u{1f336}\u{1f337}\u{1f37c}\u{1f37d}\u{1f37e}\u{1f393}\u{1f394}\u{1f39f}\u{1f3a0}\u{1f3ca}\u{1f3cb}\u{1f3ce}\u{1f3cf}\u{1f3d3}\u{1f3d4}\u{1f3df}\u{1f3e0}\u{1f3f0}\u{1f3f1}\u{1f3f3}\u{1f3f4}\u{1f3f5}\u{1f3f7}\u{1f3f8}\u{1f43e}\u{1f43f}\u{1f440}\u{1f441}\u{1f442}\u{1f4fc}\u{1f4fd}\u{1f4fe}\u{1f4ff}\u{1f53d}\u{1f53e}\u{1f54a}\u{1f54b}\u{1f54e}\u{1f54f}\u{1f550}\u{1f567}\u{1f568}\u{1f579}\u{1f57a}\u{1f57b}\u{1f594}\u{1f595}\u{1f596}\u{1f597}\u{1f5a3}\u{1f5a4}\u{1f5a5}\u{1f5fa}\u{1f5fb}\u{1f64f}\u{1f650}\u{1f67f}\u{1f680}\u{1f6c5}\u{1f6c6}\u{1f6cb}\u{1f6cc}\u{1f6cd}\u{1f6cf}\u{1f6d0}\u{1f6d2}\u{1f6d3}\u{1f6d4}\u{1f6d5}\u{1f6da}\u{1f6db}\u{1f6dc}\u{1f6df}\u{1f6e0}\u{1f6ea}\u{1f6eb}\u{1f6ec}\u{1f6ed}\u{1f6f3}\u{1f6f4}\u{1f6fc}\u{1f6fd}\u{1f7df}\u{1f7e0}\u{1f7eb}\u{1f7ec}\u{1f7ef}\u{1f7f0}\u{1f7f1}\u{1f90b}\u{1f90c}\u{1f93a}\u{1f93b}\u{1f93c}\u{1f945}\u{1f946}\u{1f947}\u{1f9ff}\u{1fa00}\u{1fa6f}\u{1fa70}\u{1fa7c}\u{1fa7d}\u{1fa7f}\u{1fa80}\u{1fac6}\u{1fac7}\u{1fac8}\u{1fac9}\u{1facb}\u{1fade}\u{1fadf}\u{1faec}\u{1faee}\u{1faef}\u{1fafb}\u{feff}ſK\n",
                "X\r\n\u{231a}\u{2028}\u{23f3}\u{2029}",
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
fn unicode_emoji_presentation_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Emoji_Presentation}}/{mode},b=/\p{{EPres}}/i{mode},n=/\P{{EPres}}/i{mode};a.test('\u231A')&&a.test('\u23F3')&&b.test('\u231A')&&!a.test('\u005f')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u231A')&&n.test('\uDB40\uDC01')&&n.test('\uD800')&&n.test('\u0020')"
        ));
        check(&format!(
            r"let p=/\p{{Emoji_Presentation}}/{mode},s=/\s/{mode};p.test('\u231A')&&p.test('\u23F3')&&!s.test('\u231A')&&!s.test('\u23F3')&&!p.test('\u0020')&&s.test('\u0020')"
        ));
        check(&format!(
            r"let a=/[\p{{EPres}}K]/i{mode},b=/[^\P{{Emoji_Presentation}}K]/i{mode},c=/[\p{{EPres}}\P{{Emoji_Presentation}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u231A')&&!a.test('ſ')&&b.test('\u23F3')&&!b.test('K')&&c.test('\uDB40\uDC01')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{EPres}}$/{mode},b=/^\P{{EPres}}$/{mode};a.test('\u231A')&&!a.test('\u23F3\n')&&!a.test('')&&!a.test('\n')&&b.test('\uDB40\uDC01')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_emoji_presentation_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{EPres}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uD83D\uDE00');m.index===0&&m[0]==='\uD83D\uDE00'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{EPres}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{EPres}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u231A');m.index===0&&m[1]==='\uDB40\uDC01'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{EPres}}/d{mode}g;r.lastIndex=1;let m=r.exec('\uDB40\uDC01\u231A');m.index===2&&m[0]==='\u231A'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{EPres}}/{mode}y;r.lastIndex=1;r.exec('\uDB40\uDC01\u231A')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_emoji_presentation_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u23F3\uDB40\uDC01\u231AG'.matchAll(/(?<x>\P{{EPres}})/d{mode}g)];a.length===2&&a[0][1]==='\uDB40\uDC01'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u23F3\u231A\uDB40\uDC01X'.replace(/(?<x>\p{{Emoji_Presentation}})/{mode}g,'<$1:$<x>>')==='<\u23F3:\u23F3><\u231A:\u231A>\uDB40\uDC01X'"
        ));
        check(&format!(
            r"'\uDB40\uDC01\u23F3\u231AG'.split(/(\p{{EPres}})/{mode}).join('|')==='\uDB40\uDC01|\u23F3||\u231A|G'&&'\uDB40\uDC01\u231A'.search(/\p{{EPres}}/{mode})===2&&'\u23F3\u231A\uDB40\uDC01'.match(/\p{{EPres}}/{mode}g).join('')==='\u23F3\u231A'"
        ));
        check(&format!(
            r"let t='';let x='\u231A\u23F3\uDB40\uDC01\u231A'.replace(/(\p{{EPres}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__\uDB40\uDC01_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_emoji_presentation_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Emoji_Presentation})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Emoji_Presentation})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u231A').groups.x==='\u231A'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{EPres}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '\uDB40\uDC01'}});t==='si'&&m[0]==='\uDB40\uDC01'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{EPres}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u231A')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{EPres}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\uDB40\uDC01\u231A')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_emoji_presentation_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{EPres}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='\\uDB40\\uDC01'.repeat(100000)+'\\n\\u231A'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u231A'&&m[3000]==='\\u231A'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_emoji_presentation_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{EPres}/ug,marker=0,text='\\uDB40\\uDC01'.repeat(10000);r.lastIndex=1")
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
        r"/\p{Script=Latin}/u.exec('A')",
        r"/[\p{EPres}&&A]/v.exec('A')",
        r"/[\p{EPres}--A]/v.exec('A')",
        r"/\p{EPres}+b/u.exec('AA')",
        r"/\p{EPres}A/v.exec('AA')",
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
fn unicode_emoji_presentation_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{EPres}})+\B/d{mode}g,m=r.exec('\uDB40\uDC01\u231A\u23F3\u23F3\u0020');m.index===2&&m[0]==='\u231A\u23F3\u23F3'&&m.groups.x==='\u23F3'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_emoji_presentation_property_unicode18_addition() {
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','u'),n=new RegExp('^\\P{Emoji_Presentation}$','u');p.test('\uD83D\uDED9')===true&&n.test('\uD83D\uDED9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','ui'),n=new RegExp('^\\P{Emoji_Presentation}$','ui');p.test('\uD83D\uDED9')===true&&n.test('\uD83D\uDED9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','v'),n=new RegExp('^\\P{Emoji_Presentation}$','v');p.test('\uD83D\uDED9')===true&&n.test('\uD83D\uDED9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','vi'),n=new RegExp('^\\P{Emoji_Presentation}$','vi');p.test('\uD83D\uDED9')===true&&n.test('\uD83D\uDED9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','u'),n=new RegExp('^\\P{EPres}$','u');p.test('\uD83D\uDED9')===true&&n.test('\uD83D\uDED9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','ui'),n=new RegExp('^\\P{EPres}$','ui');p.test('\uD83D\uDED9')===true&&n.test('\uD83D\uDED9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','v'),n=new RegExp('^\\P{EPres}$','v');p.test('\uD83D\uDED9')===true&&n.test('\uD83D\uDED9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','vi'),n=new RegExp('^\\P{EPres}$','vi');p.test('\uD83D\uDED9')===true&&n.test('\uD83D\uDED9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','u'),n=new RegExp('^\\P{Emoji_Presentation}$','u');p.test('\uD83E\uDE8B')===true&&n.test('\uD83E\uDE8B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','ui'),n=new RegExp('^\\P{Emoji_Presentation}$','ui');p.test('\uD83E\uDE8B')===true&&n.test('\uD83E\uDE8B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','v'),n=new RegExp('^\\P{Emoji_Presentation}$','v');p.test('\uD83E\uDE8B')===true&&n.test('\uD83E\uDE8B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','vi'),n=new RegExp('^\\P{Emoji_Presentation}$','vi');p.test('\uD83E\uDE8B')===true&&n.test('\uD83E\uDE8B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','u'),n=new RegExp('^\\P{EPres}$','u');p.test('\uD83E\uDE8B')===true&&n.test('\uD83E\uDE8B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','ui'),n=new RegExp('^\\P{EPres}$','ui');p.test('\uD83E\uDE8B')===true&&n.test('\uD83E\uDE8B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','v'),n=new RegExp('^\\P{EPres}$','v');p.test('\uD83E\uDE8B')===true&&n.test('\uD83E\uDE8B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','vi'),n=new RegExp('^\\P{EPres}$','vi');p.test('\uD83E\uDE8B')===true&&n.test('\uD83E\uDE8B')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','u'),n=new RegExp('^\\P{Emoji_Presentation}$','u');p.test('\uD83E\uDE8C')===true&&n.test('\uD83E\uDE8C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','ui'),n=new RegExp('^\\P{Emoji_Presentation}$','ui');p.test('\uD83E\uDE8C')===true&&n.test('\uD83E\uDE8C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','v'),n=new RegExp('^\\P{Emoji_Presentation}$','v');p.test('\uD83E\uDE8C')===true&&n.test('\uD83E\uDE8C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','vi'),n=new RegExp('^\\P{Emoji_Presentation}$','vi');p.test('\uD83E\uDE8C')===true&&n.test('\uD83E\uDE8C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','u'),n=new RegExp('^\\P{EPres}$','u');p.test('\uD83E\uDE8C')===true&&n.test('\uD83E\uDE8C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','ui'),n=new RegExp('^\\P{EPres}$','ui');p.test('\uD83E\uDE8C')===true&&n.test('\uD83E\uDE8C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','v'),n=new RegExp('^\\P{EPres}$','v');p.test('\uD83E\uDE8C')===true&&n.test('\uD83E\uDE8C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','vi'),n=new RegExp('^\\P{EPres}$','vi');p.test('\uD83E\uDE8C')===true&&n.test('\uD83E\uDE8C')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','u'),n=new RegExp('^\\P{Emoji_Presentation}$','u');p.test('\uD83E\uDE8D')===true&&n.test('\uD83E\uDE8D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','ui'),n=new RegExp('^\\P{Emoji_Presentation}$','ui');p.test('\uD83E\uDE8D')===true&&n.test('\uD83E\uDE8D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','v'),n=new RegExp('^\\P{Emoji_Presentation}$','v');p.test('\uD83E\uDE8D')===true&&n.test('\uD83E\uDE8D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','vi'),n=new RegExp('^\\P{Emoji_Presentation}$','vi');p.test('\uD83E\uDE8D')===true&&n.test('\uD83E\uDE8D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','u'),n=new RegExp('^\\P{EPres}$','u');p.test('\uD83E\uDE8D')===true&&n.test('\uD83E\uDE8D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','ui'),n=new RegExp('^\\P{EPres}$','ui');p.test('\uD83E\uDE8D')===true&&n.test('\uD83E\uDE8D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','v'),n=new RegExp('^\\P{EPres}$','v');p.test('\uD83E\uDE8D')===true&&n.test('\uD83E\uDE8D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','vi'),n=new RegExp('^\\P{EPres}$','vi');p.test('\uD83E\uDE8D')===true&&n.test('\uD83E\uDE8D')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','u'),n=new RegExp('^\\P{Emoji_Presentation}$','u');p.test('\uD83E\uDECC')===true&&n.test('\uD83E\uDECC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','ui'),n=new RegExp('^\\P{Emoji_Presentation}$','ui');p.test('\uD83E\uDECC')===true&&n.test('\uD83E\uDECC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','v'),n=new RegExp('^\\P{Emoji_Presentation}$','v');p.test('\uD83E\uDECC')===true&&n.test('\uD83E\uDECC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','vi'),n=new RegExp('^\\P{Emoji_Presentation}$','vi');p.test('\uD83E\uDECC')===true&&n.test('\uD83E\uDECC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','u'),n=new RegExp('^\\P{EPres}$','u');p.test('\uD83E\uDECC')===true&&n.test('\uD83E\uDECC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','ui'),n=new RegExp('^\\P{EPres}$','ui');p.test('\uD83E\uDECC')===true&&n.test('\uD83E\uDECC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','v'),n=new RegExp('^\\P{EPres}$','v');p.test('\uD83E\uDECC')===true&&n.test('\uD83E\uDECC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','vi'),n=new RegExp('^\\P{EPres}$','vi');p.test('\uD83E\uDECC')===true&&n.test('\uD83E\uDECC')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','u'),n=new RegExp('^\\P{Emoji_Presentation}$','u');p.test('\uD83E\uDEDD')===true&&n.test('\uD83E\uDEDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','ui'),n=new RegExp('^\\P{Emoji_Presentation}$','ui');p.test('\uD83E\uDEDD')===true&&n.test('\uD83E\uDEDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','v'),n=new RegExp('^\\P{Emoji_Presentation}$','v');p.test('\uD83E\uDEDD')===true&&n.test('\uD83E\uDEDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','vi'),n=new RegExp('^\\P{Emoji_Presentation}$','vi');p.test('\uD83E\uDEDD')===true&&n.test('\uD83E\uDEDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','u'),n=new RegExp('^\\P{EPres}$','u');p.test('\uD83E\uDEDD')===true&&n.test('\uD83E\uDEDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','ui'),n=new RegExp('^\\P{EPres}$','ui');p.test('\uD83E\uDEDD')===true&&n.test('\uD83E\uDEDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','v'),n=new RegExp('^\\P{EPres}$','v');p.test('\uD83E\uDEDD')===true&&n.test('\uD83E\uDEDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','vi'),n=new RegExp('^\\P{EPres}$','vi');p.test('\uD83E\uDEDD')===true&&n.test('\uD83E\uDEDD')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','u'),n=new RegExp('^\\P{Emoji_Presentation}$','u');p.test('\uD83E\uDEEB')===true&&n.test('\uD83E\uDEEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','ui'),n=new RegExp('^\\P{Emoji_Presentation}$','ui');p.test('\uD83E\uDEEB')===true&&n.test('\uD83E\uDEEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','v'),n=new RegExp('^\\P{Emoji_Presentation}$','v');p.test('\uD83E\uDEEB')===true&&n.test('\uD83E\uDEEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','vi'),n=new RegExp('^\\P{Emoji_Presentation}$','vi');p.test('\uD83E\uDEEB')===true&&n.test('\uD83E\uDEEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','u'),n=new RegExp('^\\P{EPres}$','u');p.test('\uD83E\uDEEB')===true&&n.test('\uD83E\uDEEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','ui'),n=new RegExp('^\\P{EPres}$','ui');p.test('\uD83E\uDEEB')===true&&n.test('\uD83E\uDEEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','v'),n=new RegExp('^\\P{EPres}$','v');p.test('\uD83E\uDEEB')===true&&n.test('\uD83E\uDEEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','vi'),n=new RegExp('^\\P{EPres}$','vi');p.test('\uD83E\uDEEB')===true&&n.test('\uD83E\uDEEB')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','u'),n=new RegExp('^\\P{Emoji_Presentation}$','u');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','ui'),n=new RegExp('^\\P{Emoji_Presentation}$','ui');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','v'),n=new RegExp('^\\P{Emoji_Presentation}$','v');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','vi'),n=new RegExp('^\\P{Emoji_Presentation}$','vi');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','u'),n=new RegExp('^\\P{EPres}$','u');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','ui'),n=new RegExp('^\\P{EPres}$','ui');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','v'),n=new RegExp('^\\P{EPres}$','v');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','vi'),n=new RegExp('^\\P{EPres}$','vi');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','u'),n=new RegExp('^\\P{Emoji_Presentation}$','u');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','ui'),n=new RegExp('^\\P{Emoji_Presentation}$','ui');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','v'),n=new RegExp('^\\P{Emoji_Presentation}$','v');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Presentation}$','vi'),n=new RegExp('^\\P{Emoji_Presentation}$','vi');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','u'),n=new RegExp('^\\P{EPres}$','u');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','ui'),n=new RegExp('^\\P{EPres}$','ui');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','v'),n=new RegExp('^\\P{EPres}$','v');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EPres}$','vi'),n=new RegExp('^\\P{EPres}$','vi');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
}
