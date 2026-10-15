//! IDS_Trinary_Operator and its exact IDST alias over complete Unicode characters.
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
fn unicode_ids_trinary_operator_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{IDS_Trinary_Operator}",
        r"\P{IDST}",
        r"[^\p{IDST}]",
        r"[\p{IDST}K]",
        r"[\P{IDS_Trinary_Operator}K]",
        r"^(?<x>[\p{IDST}K])$",
        r"(?<x>\P{IDS_Trinary_Operator})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{2ff1}\u{2ff2}\u{2ff3}\u{2ff4}\u{feff}ſK\n",
                "X\r\n\u{2ff2}\u{2028}\u{2ff3}\u{2029}",
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
fn unicode_ids_trinary_operator_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{IDS_Trinary_Operator}}/{mode},b=/\p{{IDST}}/i{mode},n=/\P{{IDST}}/i{mode};a.test('\u2FF2')&&a.test('\u2FF3')&&b.test('\u2FF2')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u2FF2')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{IDS_Trinary_Operator}}/{mode},s=/\s/{mode};p.test('\u2FF2')&&p.test('\u2FF3')&&!s.test('\u2FF2')&&!s.test('\u2FF3')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{IDST}}K]/i{mode},b=/[^\P{{IDS_Trinary_Operator}}K]/i{mode},c=/[\p{{IDST}}\P{{IDS_Trinary_Operator}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u2FF2')&&!a.test('ſ')&&b.test('\u2FF3')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{IDST}}$/{mode},b=/^\P{{IDST}}$/{mode};a.test('\u2FF2')&&!a.test('\u2FF3\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_ids_trinary_operator_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{IDST}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{IDST}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u2FF2');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{IDST}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u2FF2');m.index===2&&m[0]==='\u2FF2'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{IDST}}/{mode}y;r.lastIndex=1;r.exec('😀\u2FF2')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_ids_trinary_operator_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u2FF3😀\u2FF2G'.matchAll(/(?<x>\P{{IDST}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u2FF3\u2FF2😀X'.replace(/(?<x>\p{{IDS_Trinary_Operator}})/{mode}g,'<$1:$<x>>')==='<\u2FF3:\u2FF3><\u2FF2:\u2FF2>😀X'"
        ));
        check(&format!(
            r"'😀\u2FF3\u2FF2G'.split(/(\p{{IDST}})/{mode}).join('|')==='😀|\u2FF3||\u2FF2|G'&&'😀\u2FF2'.search(/\p{{IDST}}/{mode})===2&&'\u2FF3\u2FF2😀'.match(/\p{{IDST}}/{mode}g).join('')==='\u2FF3\u2FF2'"
        ));
        check(&format!(
            r"let t='';let x='\u2FF2\u2FF3😀\u2FF2'.replace(/(\p{{IDST}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_ids_trinary_operator_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{IDS_Trinary_Operator})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{IDS_Trinary_Operator})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u2FF2').groups.x==='\u2FF2'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{IDST}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{IDST}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u2FF2')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{IDST}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u2FF2')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_ids_trinary_operator_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{IDST}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u2FF2'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u2FF2'&&m[3000]==='\\u2FF2'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_ids_trinary_operator_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{IDST}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/[\p{IDST}&&A]/v.exec('A')",
        r"/[\p{IDST}--A]/v.exec('A')",
        r"/\p{IDST}+b/u.exec('AA')",
        r"/\p{IDST}A/v.exec('AA')",
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
fn unicode_ids_trinary_operator_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{IDST}})+\B/d{mode}g,m=r.exec('😀\u2FF2\u2FF3\u2FF3\uFEFF');m.index===2&&m[0]==='\u2FF2\u2FF3\u2FF3'&&m.groups.x==='\u2FF3'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}
