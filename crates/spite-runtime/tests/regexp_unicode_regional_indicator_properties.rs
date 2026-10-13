//! Regional_Indicator and its exact RI alias over complete Unicode characters.
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
fn unicode_regional_indicator_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Regional_Indicator}",
        r"\P{RI}",
        r"[^\p{RI}]",
        r"[\p{RI}K]",
        r"[\P{Regional_Indicator}K]",
        r"^(?<x>[\p{RI}K])$",
        r"(?<x>\P{Regional_Indicator})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{1f1e5}\u{1f1e6}\u{1f1ff}\u{1f200}\u{feff}ſK\n",
                "X\r\n\u{1f1e6}\u{2028}\u{1f1ff}\u{2029}",
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
fn unicode_regional_indicator_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Regional_Indicator}}/{mode},b=/\p{{RI}}/i{mode},n=/\P{{RI}}/i{mode};a.test('\u{{1F1E6}}')&&a.test('\u{{1F1FF}}')&&b.test('\u{{1F1E6}}')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u{{1F1E6}}')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Regional_Indicator}}/{mode},s=/\s/{mode};p.test('\u{{1F1E6}}')&&p.test('\u{{1F1FF}}')&&!s.test('\u{{1F1E6}}')&&!s.test('\u{{1F1FF}}')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{RI}}K]/i{mode},b=/[^\P{{Regional_Indicator}}K]/i{mode},c=/[\p{{RI}}\P{{Regional_Indicator}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u{{1F1E6}}')&&!a.test('ſ')&&b.test('\u{{1F1FF}}')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{RI}}$/{mode},b=/^\P{{RI}}$/{mode};a.test('\u{{1F1E6}}')&&!a.test('\u{{1F1FF}}\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_regional_indicator_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{RI}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{RI}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u{{1F1E6}}');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{RI}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u{{1F1E6}}');m.index===2&&m[0]==='\u{{1F1E6}}'&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/\p{{RI}}/{mode}y;r.lastIndex=1;r.exec('😀\u{{1F1E6}}')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_regional_indicator_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u{{1F1FF}}😀\u{{1F1E6}}G'.matchAll(/(?<x>\P{{RI}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===2&&a[0].indices[1][1]===4&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u{{1F1FF}}\u{{1F1E6}}😀X'.replace(/(?<x>\p{{Regional_Indicator}})/{mode}g,'<$1:$<x>>')==='<\u{{1F1FF}}:\u{{1F1FF}}><\u{{1F1E6}}:\u{{1F1E6}}>😀X'"
        ));
        check(&format!(
            r"'😀\u{{1F1FF}}\u{{1F1E6}}G'.split(/(\p{{RI}})/{mode}).join('|')==='😀|\u{{1F1FF}}||\u{{1F1E6}}|G'&&'😀\u{{1F1E6}}'.search(/\p{{RI}}/{mode})===2&&'\u{{1F1FF}}\u{{1F1E6}}😀'.match(/\p{{RI}}/{mode}g).join('')==='\u{{1F1FF}}\u{{1F1E6}}'"
        ));
        check(&format!(
            r"let t='';let x='\u{{1F1E6}}\u{{1F1FF}}😀\u{{1F1E6}}'.replace(/(\p{{RI}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='026'"
        ));
    }
}

#[test]
fn unicode_regional_indicator_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Regional_Indicator})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Regional_Indicator})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u{1F1E6}').groups.x==='\u{1F1E6}'&&c.lastIndex===2",
    );
    check(
        r"let t='',r=/\P{RI}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{RI}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u{1F1E6}')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{RI}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u{1F1E6}')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_regional_indicator_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{RI}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u{1F1E6}'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u{1F1E6}'&&m[3000]==='\\u{1F1E6}'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200003&&c.lastIndex===200003"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_regional_indicator_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{RI}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/[\p{RI}&&A]/v.exec('A')",
        r"/[\p{RI}--A]/v.exec('A')",
        r"/\p{RI}+b/u.exec('AA')",
        r"/\p{RI}A/v.exec('AA')",
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
fn unicode_regional_indicator_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{RI}})+\B/d{mode}g,m=r.exec('😀\u{{1F1E6}}\u{{1F1FF}}\u{{1F1FF}}\uFEFF');m.index===2&&m[0]==='\u{{1F1E6}}\u{{1F1FF}}\u{{1F1FF}}'&&m.groups.x==='\u{{1F1FF}}'&&m.indices.groups.x[0]===6&&m.indices.groups.x[1]===8&&r.lastIndex===8"
        ));
    }
}

#[test]
fn unicode_regional_indicator_property_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{RI}})/d{mode}y;r.lastIndex=1;let m=r.exec('\u{{1F1E6}}');m.index===0&&m.groups.x==='\u{{1F1E6}}'&&m.indices.groups.x[0]===0&&m.indices.groups.x[1]===2&&r.lastIndex===2"
        ));
    }
}
