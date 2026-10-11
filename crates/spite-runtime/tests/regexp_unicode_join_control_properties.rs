//! Join_Control and its exact Join_C alias over complete Unicode characters.
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
fn unicode_join_control_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Join_Control}",
        r"\P{Join_C}",
        r"[^\p{Join_C}]",
        r"[\p{Join_C}K]",
        r"[\P{Join_Control}K]",
        r"^(?<x>[\p{Join_C}K])$",
        r"(?<x>\P{Join_Control})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{200b}\u{200c}\u{200d}\u{200e}ſK\n",
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
fn unicode_join_control_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Join_Control}}/{mode},b=/\p{{Join_C}}/i{mode},n=/\P{{Join_C}}/i{mode};a.test('\u200C')&&a.test('\u200D')&&b.test('\u200C')&&!a.test('\u200B')&&!b.test('\u200E')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u200C')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Join_Control}}/{mode},s=/\s/{mode};p.test('\u200C')&&p.test('\u200D')&&!s.test('\u200C')&&!s.test('\u200D')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Join_C}}K]/i{mode},b=/[^\P{{Join_Control}}K]/i{mode},c=/[\p{{Join_C}}\P{{Join_Control}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u200C')&&!a.test('ſ')&&b.test('\u200D')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Join_C}}$/{mode},b=/^\P{{Join_C}}$/{mode};a.test('\u200C')&&!a.test('\u200D\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_join_control_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Join_C}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Join_C}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u200c');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Join_C}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u200c');m.index===2&&m[0]==='\u200c'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Join_C}}/{mode}y;r.lastIndex=1;r.exec('😀\u200c')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_join_control_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u200d😀\u200cG'.matchAll(/(?<x>\P{{Join_C}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u200d\u200c😀X'.replace(/(?<x>\p{{Join_Control}})/{mode}g,'<$1:$<x>>')==='<\u200d:\u200d><\u200c:\u200c>😀X'"
        ));
        check(&format!(
            r"'😀\u200d\u200cG'.split(/(\p{{Join_C}})/{mode}).join('|')==='😀|\u200d||\u200c|G'&&'😀\u200c'.search(/\p{{Join_C}}/{mode})===2&&'\u200d\u200c😀'.match(/\p{{Join_C}}/{mode}g).join('')==='\u200d\u200c'"
        ));
        check(&format!(
            r"let t='';let x='\u200c\u200d😀\u200c'.replace(/(\p{{Join_C}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_join_control_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Join_Control})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Join_Control})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u200c').groups.x==='\u200c'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Join_C}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Join_C}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u200c')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Join_C}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u200c')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_join_control_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{Join_C}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u200c'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u200c'&&m[3000]==='\\u200c'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_join_control_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Join_C}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/[\p{Join_C}&&A]/v.exec('A')",
        r"/[\p{Join_C}--A]/v.exec('A')",
        r"/\p{Join_C}+b/u.exec('AA')",
        r"/\p{Join_C}A/v.exec('AA')",
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
