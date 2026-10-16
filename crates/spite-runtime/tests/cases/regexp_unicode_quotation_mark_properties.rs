//! Quotation_Mark and its exact QMark alias over complete Unicode characters.
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
fn unicode_quotation_mark_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Quotation_Mark}",
        r"\P{QMark}",
        r"[^\p{QMark}]",
        r"[\p{QMark}K]",
        r"[\P{Quotation_Mark}K]",
        r"^(?<x>[\p{QMark}K])$",
        r"(?<x>\P{Quotation_Mark})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{21}\u{22}\u{23}\u{26}\u{27}\u{28}\u{ab}\u{bb}\u{2017}\u{2018}\u{2019}\u{201a}\u{201b}\u{201c}\u{201d}\u{201e}\u{201f}\u{2039}\u{203a}\u{2e42}\u{300c}\u{300d}\u{300e}\u{300f}\u{301d}\u{301e}\u{301f}\u{fe41}\u{fe42}\u{fe43}\u{fe44}\u{ff02}\u{ff07}\u{ff62}\u{ff63}\u{2010}\u{300b}\u{3010}\u{ff64}ſK\n",
                "X\r\n\u{201c}\u{2028}\u{301f}\u{2029}",
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
fn unicode_quotation_mark_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Quotation_Mark}}/{mode},b=/\p{{QMark}}/i{mode},n=/\P{{QMark}}/i{mode};a.test('\u201C')&&a.test('\u201D')&&b.test('\u201C')&&!a.test('\u200D')&&!b.test('\u2010')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u201C')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Quotation_Mark}}/{mode},s=/\s/{mode};p.test('\u201C')&&p.test('\u201D')&&!s.test('\u201C')&&!s.test('\u201D')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{QMark}}K]/i{mode},b=/[^\P{{Quotation_Mark}}K]/i{mode},c=/[\p{{QMark}}\P{{Quotation_Mark}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u201C')&&!a.test('ſ')&&b.test('\u201D')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{QMark}}$/{mode},b=/^\P{{QMark}}$/{mode};a.test('\u201C')&&!a.test('\u201D\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_quotation_mark_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{QMark}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{QMark}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u201c');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{QMark}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u201c');m.index===2&&m[0]==='\u201c'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{QMark}}/{mode}y;r.lastIndex=1;r.exec('😀\u201c')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_quotation_mark_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u201d😀\u201cG'.matchAll(/(?<x>\P{{QMark}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u201d\u201c😀X'.replace(/(?<x>\p{{Quotation_Mark}})/{mode}g,'<$1:$<x>>')==='<\u201d:\u201d><\u201c:\u201c>😀X'"
        ));
        check(&format!(
            r"'😀\u201d\u201cG'.split(/(\p{{QMark}})/{mode}).join('|')==='😀|\u201d||\u201c|G'&&'😀\u201c'.search(/\p{{QMark}}/{mode})===2&&'\u201d\u201c😀'.match(/\p{{QMark}}/{mode}g).join('')==='\u201d\u201c'"
        ));
        check(&format!(
            r"let t='';let x='\u201c\u201d😀\u201c'.replace(/(\p{{QMark}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_quotation_mark_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Quotation_Mark})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Quotation_Mark})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u201c').groups.x==='\u201c'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{QMark}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{QMark}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u201c')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{QMark}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u201c')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_quotation_mark_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{QMark}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u201c'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u201c'&&m[3000]==='\\u201c'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_quotation_mark_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{QMark}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    check(r"/[\p{QMark}&&A]/v.exec('A')===null");
    check(r"/[\p{QMark}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x201c),m=/[\p{QMark}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [r"/\p{QMark}+b/u.exec('AA')", r"/\p{QMark}A/v.exec('AA')"] {
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
fn unicode_quotation_mark_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{QMark}})+\B/d{mode}g,m=r.exec('😀\u201c\u201d\u301f!');m.index===2&&m[0]==='\u201c\u201d\u301f'&&m.groups.x==='\u301f'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}
