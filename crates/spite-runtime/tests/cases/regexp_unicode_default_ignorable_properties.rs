//! Default_Ignorable_Code_Point and its exact DI alias over complete Unicode characters.
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
fn unicode_default_ignorable_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Default_Ignorable_Code_Point}",
        r"\P{DI}",
        r"[^\p{DI}]",
        r"[\p{DI}K]",
        r"[\P{Default_Ignorable_Code_Point}K]",
        r"^(?<x>[\p{DI}K])$",
        r"(?<x>\P{Default_Ignorable_Code_Point})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{ac}\u{ad}\u{ae}\u{34e}\u{34f}\u{350}\u{61b}\u{61c}\u{61d}\u{115e}\u{115f}\u{1160}\u{1161}\u{17b3}\u{17b4}\u{17b5}\u{17b6}\u{180a}\u{180b}\u{180f}\u{1810}\u{200a}\u{200b}\u{200f}\u{2010}\u{2029}\u{202a}\u{202e}\u{202f}\u{205f}\u{2060}\u{206f}\u{2070}\u{3163}\u{3164}\u{3165}\u{fdff}\u{fe00}\u{fe0f}\u{fe10}\u{fefe}\u{feff}\u{ff00}\u{ff9f}\u{ffa0}\u{ffa1}\u{ffef}\u{fff0}\u{fff8}\u{fff9}\u{1bc9f}\u{1bca0}\u{1bca3}\u{1bca4}\u{1d172}\u{1d173}\u{1d17a}\u{1d17b}\u{dffff}\u{e0000}\u{e0fff}\u{e1000}\u{feff}ſK\n",
                "X\r\n\u{200c}\u{2028}\u{200d}\u{2029}",
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
fn unicode_default_ignorable_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Default_Ignorable_Code_Point}}/{mode},b=/\p{{DI}}/i{mode},n=/\P{{DI}}/i{mode};a.test('\u200C')&&a.test('\u200D')&&b.test('\u200C')&&!a.test('\u005f')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u200C')&&n.test('😀')&&n.test('\uD800')&&n.test('\u0020')"
        ));
        check(&format!(
            r"let p=/\p{{Default_Ignorable_Code_Point}}/{mode},s=/\s/{mode};p.test('\u200C')&&p.test('\u200D')&&!s.test('\u200C')&&!s.test('\u200D')&&!p.test('\u0020')&&s.test('\u0020')"
        ));
        check(&format!(
            r"let a=/[\p{{DI}}K]/i{mode},b=/[^\P{{Default_Ignorable_Code_Point}}K]/i{mode},c=/[\p{{DI}}\P{{Default_Ignorable_Code_Point}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u200C')&&!a.test('ſ')&&b.test('\u200D')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{DI}}$/{mode},b=/^\P{{DI}}$/{mode};a.test('\u200C')&&!a.test('\u200D\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_default_ignorable_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{DI}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uDB40\uDC01');m.index===0&&m[0]==='\uDB40\uDC01'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }

    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{DI}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{DI}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u200C');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{DI}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u200C');m.index===2&&m[0]==='\u200C'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{DI}}/{mode}y;r.lastIndex=1;r.exec('😀\u200C')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_default_ignorable_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u200D😀\u200CG'.matchAll(/(?<x>\P{{DI}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u200D\u200C😀X'.replace(/(?<x>\p{{Default_Ignorable_Code_Point}})/{mode}g,'<$1:$<x>>')==='<\u200D:\u200D><\u200C:\u200C>😀X'"
        ));
        check(&format!(
            r"'😀\u200D\u200CG'.split(/(\p{{DI}})/{mode}).join('|')==='😀|\u200D||\u200C|G'&&'😀\u200C'.search(/\p{{DI}}/{mode})===2&&'\u200D\u200C😀'.match(/\p{{DI}}/{mode}g).join('')==='\u200D\u200C'"
        ));
        check(&format!(
            r"let t='';let x='\u200C\u200D😀\u200C'.replace(/(\p{{DI}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_default_ignorable_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Default_Ignorable_Code_Point})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Default_Ignorable_Code_Point})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u200C').groups.x==='\u200C'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{DI}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{DI}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u200C')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{DI}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u200C')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_default_ignorable_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{DI}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u200C'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u200C'&&m[3000]==='\\u200C'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_default_ignorable_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{DI}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/[\p{DI}&&A]/v.exec('A')",
        r"/[\p{DI}--A]/v.exec('A')",
        r"/\p{DI}+b/u.exec('AA')",
        r"/\p{DI}A/v.exec('AA')",
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
fn unicode_default_ignorable_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{DI}})+\B/d{mode}g,m=r.exec('😀\u200C\u200D\u200D\u0020');m.index===2&&m[0]==='\u200C\u200D\u200D'&&m.groups.x==='\u200D'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}
