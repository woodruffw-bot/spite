//! Emoji_Modifier_Base and its exact EBase alias over complete Unicode characters.
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
fn unicode_emoji_modifier_base_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Emoji_Modifier_Base}",
        r"\P{EBase}",
        r"[^\p{EBase}]",
        r"[\p{EBase}K]",
        r"[\P{Emoji_Modifier_Base}K]",
        r"^(?<x>[\p{EBase}K])$",
        r"(?<x>\P{Emoji_Modifier_Base})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{261c}\u{261d}\u{261e}\u{26f8}\u{26f9}\u{26fa}\u{2709}\u{270a}\u{270d}\u{270e}\u{1f384}\u{1f385}\u{1f386}\u{1f3c1}\u{1f3c2}\u{1f3c4}\u{1f3c5}\u{1f3c6}\u{1f3c7}\u{1f3c8}\u{1f3c9}\u{1f3ca}\u{1f3cc}\u{1f3cd}\u{1f441}\u{1f442}\u{1f443}\u{1f444}\u{1f445}\u{1f446}\u{1f450}\u{1f451}\u{1f465}\u{1f466}\u{1f478}\u{1f479}\u{1f47b}\u{1f47c}\u{1f47d}\u{1f480}\u{1f481}\u{1f483}\u{1f484}\u{1f485}\u{1f487}\u{1f488}\u{1f48e}\u{1f48f}\u{1f490}\u{1f491}\u{1f492}\u{1f4a9}\u{1f4aa}\u{1f4ab}\u{1f573}\u{1f574}\u{1f575}\u{1f576}\u{1f579}\u{1f57a}\u{1f57b}\u{1f58f}\u{1f590}\u{1f591}\u{1f594}\u{1f595}\u{1f596}\u{1f597}\u{1f644}\u{1f645}\u{1f647}\u{1f648}\u{1f64a}\u{1f64b}\u{1f64f}\u{1f650}\u{1f6a2}\u{1f6a3}\u{1f6a4}\u{1f6b3}\u{1f6b4}\u{1f6b6}\u{1f6b7}\u{1f6bf}\u{1f6c0}\u{1f6c1}\u{1f6cb}\u{1f6cc}\u{1f6cd}\u{1f90b}\u{1f90c}\u{1f90d}\u{1f90e}\u{1f90f}\u{1f910}\u{1f917}\u{1f918}\u{1f91f}\u{1f920}\u{1f925}\u{1f926}\u{1f927}\u{1f92f}\u{1f930}\u{1f939}\u{1f93a}\u{1f93b}\u{1f93c}\u{1f93e}\u{1f93f}\u{1f976}\u{1f977}\u{1f978}\u{1f9b4}\u{1f9b5}\u{1f9b6}\u{1f9b7}\u{1f9b8}\u{1f9b9}\u{1f9ba}\u{1f9bb}\u{1f9bc}\u{1f9cc}\u{1f9cd}\u{1f9cf}\u{1f9d0}\u{1f9d1}\u{1f9dd}\u{1f9de}\u{1fac2}\u{1fac3}\u{1fac5}\u{1fac6}\u{1faef}\u{1faf0}\u{1fafb}\u{feff}ſK\n",
                "X\r\n\u{261d}\u{2028}\u{270a}\u{2029}",
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
fn unicode_emoji_modifier_base_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Emoji_Modifier_Base}}/{mode},b=/\p{{EBase}}/i{mode},n=/\P{{EBase}}/i{mode};a.test('\u261D')&&a.test('\u270A')&&b.test('\u261D')&&!a.test('\u005f')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u261D')&&n.test('😀')&&n.test('\uD800')&&n.test('\u0020')"
        ));
        check(&format!(
            r"let p=/\p{{Emoji_Modifier_Base}}/{mode},s=/\s/{mode};p.test('\u261D')&&p.test('\u270A')&&!s.test('\u261D')&&!s.test('\u270A')&&!p.test('\u0020')&&s.test('\u0020')"
        ));
        check(&format!(
            r"let a=/[\p{{EBase}}K]/i{mode},b=/[^\P{{Emoji_Modifier_Base}}K]/i{mode},c=/[\p{{EBase}}\P{{Emoji_Modifier_Base}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u261D')&&!a.test('ſ')&&b.test('\u270A')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{EBase}}$/{mode},b=/^\P{{EBase}}$/{mode};a.test('\u261D')&&!a.test('\u270A\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_emoji_modifier_base_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{EBase}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uD83C\uDFC3');m.index===0&&m[0]==='\uD83C\uDFC3'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{EBase}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{EBase}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u261D');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{EBase}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u261D');m.index===2&&m[0]==='\u261D'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{EBase}}/{mode}y;r.lastIndex=1;r.exec('😀\u261D')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_emoji_modifier_base_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u270A😀\u261DG'.matchAll(/(?<x>\P{{EBase}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u270A\u261D😀X'.replace(/(?<x>\p{{Emoji_Modifier_Base}})/{mode}g,'<$1:$<x>>')==='<\u270A:\u270A><\u261D:\u261D>😀X'"
        ));
        check(&format!(
            r"'😀\u270A\u261DG'.split(/(\p{{EBase}})/{mode}).join('|')==='😀|\u270A||\u261D|G'&&'😀\u261D'.search(/\p{{EBase}}/{mode})===2&&'\u270A\u261D😀'.match(/\p{{EBase}}/{mode}g).join('')==='\u270A\u261D'"
        ));
        check(&format!(
            r"let t='';let x='\u261D\u270A😀\u261D'.replace(/(\p{{EBase}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_emoji_modifier_base_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Emoji_Modifier_Base})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Emoji_Modifier_Base})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u261D').groups.x==='\u261D'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{EBase}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{EBase}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u261D')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{EBase}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u261D')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_emoji_modifier_base_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{EBase}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u261D'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u261D'&&m[3000]==='\\u261D'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_emoji_modifier_base_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{EBase}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/[\p{EBase}&&A]/v.exec('A')",
        r"/[\p{EBase}--A]/v.exec('A')",
        r"/\p{EBase}+b/u.exec('AA')",
        r"/\p{EBase}A/v.exec('AA')",
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
fn unicode_emoji_modifier_base_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{EBase}})+\B/d{mode}g,m=r.exec('😀\u261D\u270A\u270A\u0020');m.index===2&&m[0]==='\u261D\u270A\u270A'&&m.groups.x==='\u270A'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_emoji_modifier_base_property_unicode18_addition() {
    check(
        r"let p=new RegExp('^\\p{Emoji_Modifier_Base}$','u'),n=new RegExp('^\\P{Emoji_Modifier_Base}$','u');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Modifier_Base}$','ui'),n=new RegExp('^\\P{Emoji_Modifier_Base}$','ui');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Modifier_Base}$','v'),n=new RegExp('^\\P{Emoji_Modifier_Base}$','v');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Modifier_Base}$','vi'),n=new RegExp('^\\P{Emoji_Modifier_Base}$','vi');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EBase}$','u'),n=new RegExp('^\\P{EBase}$','u');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EBase}$','ui'),n=new RegExp('^\\P{EBase}$','ui');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EBase}$','v'),n=new RegExp('^\\P{EBase}$','v');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EBase}$','vi'),n=new RegExp('^\\P{EBase}$','vi');p.test('\uD83E\uDEF9')===true&&n.test('\uD83E\uDEF9')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Modifier_Base}$','u'),n=new RegExp('^\\P{Emoji_Modifier_Base}$','u');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Modifier_Base}$','ui'),n=new RegExp('^\\P{Emoji_Modifier_Base}$','ui');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Modifier_Base}$','v'),n=new RegExp('^\\P{Emoji_Modifier_Base}$','v');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{Emoji_Modifier_Base}$','vi'),n=new RegExp('^\\P{Emoji_Modifier_Base}$','vi');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EBase}$','u'),n=new RegExp('^\\P{EBase}$','u');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EBase}$','ui'),n=new RegExp('^\\P{EBase}$','ui');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EBase}$','v'),n=new RegExp('^\\P{EBase}$','v');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
    check(
        r"let p=new RegExp('^\\p{EBase}$','vi'),n=new RegExp('^\\P{EBase}$','vi');p.test('\uD83E\uDEFA')===true&&n.test('\uD83E\uDEFA')===false",
    );
}
