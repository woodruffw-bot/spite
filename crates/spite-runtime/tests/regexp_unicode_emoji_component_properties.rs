//! Emoji_Component and its exact EComp alias over complete Unicode characters.
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
fn unicode_emoji_component_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Emoji_Component}",
        r"\P{EComp}",
        r"[^\p{EComp}]",
        r"[\p{EComp}K]",
        r"[\P{Emoji_Component}K]",
        r"^(?<x>[\p{EComp}K])$",
        r"(?<x>\P{Emoji_Component})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{22}\u{23}\u{24}\u{29}\u{2a}\u{2b}\u{2f}\u{30}\u{39}\u{3a}\u{200c}\u{200d}\u{200d}\u{20e2}\u{20e3}\u{20e4}\u{fe0e}\u{fe0f}\u{fe10}\u{1f1e5}\u{1f1e6}\u{1f1ff}\u{1f200}\u{1f3fa}\u{1f3fb}\u{1f3ff}\u{1f400}\u{1f9af}\u{1f9b0}\u{1f9b3}\u{1f9b4}\u{e001f}\u{e0020}\u{e007f}\u{e0080}\u{feff}ſK\n",
                "X\r\n\u{200d}\u{2028}\u{fe0f}\u{2029}",
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
fn unicode_emoji_component_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Emoji_Component}}/{mode},b=/\p{{EComp}}/i{mode},n=/\P{{EComp}}/i{mode};a.test('\u200D')&&a.test('\uFE0F')&&b.test('\u200D')&&!a.test('\u005f')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u200D')&&n.test('😀')&&n.test('\uD800')&&n.test('\u0020')"
        ));
        check(&format!(
            r"let p=/\p{{Emoji_Component}}/{mode},s=/\s/{mode};p.test('\u200D')&&p.test('\uFE0F')&&!s.test('\u200D')&&!s.test('\uFE0F')&&!p.test('\u0020')&&s.test('\u0020')"
        ));
        check(&format!(
            r"let a=/[\p{{EComp}}K]/i{mode},b=/[^\P{{Emoji_Component}}K]/i{mode},c=/[\p{{EComp}}\P{{Emoji_Component}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u200D')&&!a.test('ſ')&&b.test('\uFE0F')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{EComp}}$/{mode},b=/^\P{{EComp}}$/{mode};a.test('\u200D')&&!a.test('\uFE0F\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_emoji_component_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{EComp}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uD83C\uDFFB');m.index===0&&m[0]==='\uD83C\uDFFB'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{EComp}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{EComp}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u200D');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{EComp}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u200D');m.index===2&&m[0]==='\u200D'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{EComp}}/{mode}y;r.lastIndex=1;r.exec('😀\u200D')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_emoji_component_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\uFE0F😀\u200DG'.matchAll(/(?<x>\P{{EComp}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\uFE0F\u200D😀X'.replace(/(?<x>\p{{Emoji_Component}})/{mode}g,'<$1:$<x>>')==='<\uFE0F:\uFE0F><\u200D:\u200D>😀X'"
        ));
        check(&format!(
            r"'😀\uFE0F\u200DG'.split(/(\p{{EComp}})/{mode}).join('|')==='😀|\uFE0F||\u200D|G'&&'😀\u200D'.search(/\p{{EComp}}/{mode})===2&&'\uFE0F\u200D😀'.match(/\p{{EComp}}/{mode}g).join('')==='\uFE0F\u200D'"
        ));
        check(&format!(
            r"let t='';let x='\u200D\uFE0F😀\u200D'.replace(/(\p{{EComp}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_emoji_component_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Emoji_Component})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Emoji_Component})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u200D').groups.x==='\u200D'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{EComp}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{EComp}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u200D')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{EComp}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u200D')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_emoji_component_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{EComp}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u200D'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u200D'&&m[3000]==='\\u200D'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_emoji_component_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{EComp}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/\p{Assigned}/u.exec('A')",
        r"/[\p{EComp}&&A]/v.exec('A')",
        r"/[\p{EComp}--A]/v.exec('A')",
        r"/\p{EComp}+b/u.exec('AA')",
        r"/\p{EComp}A/v.exec('AA')",
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
fn unicode_emoji_component_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{EComp}})+\B/d{mode}g,m=r.exec('😀\u200D\uFE0F\uFE0F\u0020');m.index===2&&m[0]==='\u200D\uFE0F\uFE0F'&&m.groups.x==='\uFE0F'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}
