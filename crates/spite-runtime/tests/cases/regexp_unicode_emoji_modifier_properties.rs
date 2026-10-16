//! Emoji_Modifier and its exact EMod alias over complete Unicode characters.
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
fn unicode_emoji_modifier_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Emoji_Modifier}",
        r"\P{EMod}",
        r"[^\p{EMod}]",
        r"[\p{EMod}K]",
        r"[\P{Emoji_Modifier}K]",
        r"^(?<x>[\p{EMod}K])$",
        r"(?<x>\P{Emoji_Modifier})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{1f3fa}\u{1f3fb}\u{1f3ff}\u{1f400}\u{feff}ſK\n",
                "X\r\n\u{1f3fb}\u{2028}\u{1f3ff}\u{2029}",
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
fn unicode_emoji_modifier_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Emoji_Modifier}}/{mode},b=/\p{{EMod}}/i{mode},n=/\P{{EMod}}/i{mode};a.test('\u{{1F3FB}}')&&a.test('\u{{1F3FF}}')&&b.test('\u{{1F3FB}}')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u{{1F3FB}}')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Emoji_Modifier}}/{mode},s=/\s/{mode};p.test('\u{{1F3FB}}')&&p.test('\u{{1F3FF}}')&&!s.test('\u{{1F3FB}}')&&!s.test('\u{{1F3FF}}')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{EMod}}K]/i{mode},b=/[^\P{{Emoji_Modifier}}K]/i{mode},c=/[\p{{EMod}}\P{{Emoji_Modifier}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u{{1F3FB}}')&&!a.test('ſ')&&b.test('\u{{1F3FF}}')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{EMod}}$/{mode},b=/^\P{{EMod}}$/{mode};a.test('\u{{1F3FB}}')&&!a.test('\u{{1F3FF}}\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_emoji_modifier_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{EMod}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{EMod}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u{{1F3FB}}');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{EMod}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u{{1F3FB}}');m.index===2&&m[0]==='\u{{1F3FB}}'&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/\p{{EMod}}/{mode}y;r.lastIndex=1;r.exec('😀\u{{1F3FB}}')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_emoji_modifier_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u{{1F3FF}}😀\u{{1F3FB}}G'.matchAll(/(?<x>\P{{EMod}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===2&&a[0].indices[1][1]===4&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u{{1F3FF}}\u{{1F3FB}}😀X'.replace(/(?<x>\p{{Emoji_Modifier}})/{mode}g,'<$1:$<x>>')==='<\u{{1F3FF}}:\u{{1F3FF}}><\u{{1F3FB}}:\u{{1F3FB}}>😀X'"
        ));
        check(&format!(
            r"'😀\u{{1F3FF}}\u{{1F3FB}}G'.split(/(\p{{EMod}})/{mode}).join('|')==='😀|\u{{1F3FF}}||\u{{1F3FB}}|G'&&'😀\u{{1F3FB}}'.search(/\p{{EMod}}/{mode})===2&&'\u{{1F3FF}}\u{{1F3FB}}😀'.match(/\p{{EMod}}/{mode}g).join('')==='\u{{1F3FF}}\u{{1F3FB}}'"
        ));
        check(&format!(
            r"let t='';let x='\u{{1F3FB}}\u{{1F3FF}}😀\u{{1F3FB}}'.replace(/(\p{{EMod}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='026'"
        ));
    }
}

#[test]
fn unicode_emoji_modifier_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Emoji_Modifier})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Emoji_Modifier})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u{1F3FB}').groups.x==='\u{1F3FB}'&&c.lastIndex===2",
    );
    check(
        r"let t='',r=/\P{EMod}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{EMod}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u{1F3FB}')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{EMod}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u{1F3FB}')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_emoji_modifier_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{EMod}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u{1F3FB}'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u{1F3FB}'&&m[3000]==='\\u{1F3FB}'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200003&&c.lastIndex===200003"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_emoji_modifier_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{EMod}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/[\p{EMod}&&A]/v.exec('A')",
        r"/[\p{EMod}--A]/v.exec('A')",
        r"/\p{EMod}+b/u.exec('AA')",
        r"/\p{EMod}A/v.exec('AA')",
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
fn unicode_emoji_modifier_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{EMod}})+\B/d{mode}g,m=r.exec('😀\u{{1F3FB}}\u{{1F3FF}}\u{{1F3FF}}\uFEFF');m.index===2&&m[0]==='\u{{1F3FB}}\u{{1F3FF}}\u{{1F3FF}}'&&m.groups.x==='\u{{1F3FF}}'&&m.indices.groups.x[0]===6&&m.indices.groups.x[1]===8&&r.lastIndex===8"
        ));
    }
}

#[test]
fn unicode_emoji_modifier_property_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{EMod}})/d{mode}y;r.lastIndex=1;let m=r.exec('\u{{1F3FB}}');m.index===0&&m.groups.x==='\u{{1F3FB}}'&&m.indices.groups.x[0]===0&&m.indices.groups.x[1]===2&&r.lastIndex===2"
        ));
    }
}
