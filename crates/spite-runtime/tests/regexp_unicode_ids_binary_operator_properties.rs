//! IDS_Binary_Operator and its exact IDSB alias over complete Unicode characters.
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
fn unicode_ids_binary_operator_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{IDS_Binary_Operator}",
        r"\P{IDSB}",
        r"[^\p{IDSB}]",
        r"[\p{IDSB}K]",
        r"[\P{IDS_Binary_Operator}K]",
        r"^(?<x>[\p{IDSB}K])$",
        r"(?<x>\P{IDS_Binary_Operator})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{2fef}\u{2ff0}\u{2ff1}\u{2ff2}\u{2ff3}\u{2ff4}\u{2ffd}\u{2ffe}\u{31ee}\u{31ef}\u{31f0}\u{feff}ſK\n",
                "X\r\n\u{2ff0}\u{2028}\u{31ef}\u{2029}",
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
fn unicode_ids_binary_operator_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{IDS_Binary_Operator}}/{mode},b=/\p{{IDSB}}/i{mode},n=/\P{{IDSB}}/i{mode};a.test('\u2FF0')&&a.test('\u2FF4')&&b.test('\u2FF0')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u2FF0')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{IDS_Binary_Operator}}/{mode},s=/\s/{mode};p.test('\u2FF0')&&p.test('\u2FF4')&&!s.test('\u2FF0')&&!s.test('\u2FF4')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{IDSB}}K]/i{mode},b=/[^\P{{IDS_Binary_Operator}}K]/i{mode},c=/[\p{{IDSB}}\P{{IDS_Binary_Operator}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u2FF0')&&!a.test('ſ')&&b.test('\u2FF4')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{IDSB}}$/{mode},b=/^\P{{IDSB}}$/{mode};a.test('\u2FF0')&&!a.test('\u2FF4\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_ids_binary_operator_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{IDSB}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{IDSB}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u2FF0');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{IDSB}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u2FF0');m.index===2&&m[0]==='\u2FF0'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{IDSB}}/{mode}y;r.lastIndex=1;r.exec('😀\u2FF0')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_ids_binary_operator_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u2FF4😀\u2FF0G'.matchAll(/(?<x>\P{{IDSB}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u2FF4\u2FF0😀X'.replace(/(?<x>\p{{IDS_Binary_Operator}})/{mode}g,'<$1:$<x>>')==='<\u2FF4:\u2FF4><\u2FF0:\u2FF0>😀X'"
        ));
        check(&format!(
            r"'😀\u2FF4\u2FF0G'.split(/(\p{{IDSB}})/{mode}).join('|')==='😀|\u2FF4||\u2FF0|G'&&'😀\u2FF0'.search(/\p{{IDSB}}/{mode})===2&&'\u2FF4\u2FF0😀'.match(/\p{{IDSB}}/{mode}g).join('')==='\u2FF4\u2FF0'"
        ));
        check(&format!(
            r"let t='';let x='\u2FF0\u2FF4😀\u2FF0'.replace(/(\p{{IDSB}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_ids_binary_operator_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{IDS_Binary_Operator})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{IDS_Binary_Operator})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u2FF0').groups.x==='\u2FF0'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{IDSB}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{IDSB}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u2FF0')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{IDSB}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u2FF0')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_ids_binary_operator_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{IDSB}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u2FF0'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u2FF0'&&m[3000]==='\\u2FF0'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_ids_binary_operator_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{IDSB}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/[\p{IDSB}&&A]/v.exec('A')",
        r"/[\p{IDSB}--A]/v.exec('A')",
        r"/\p{IDSB}+b/u.exec('AA')",
        r"/\p{IDSB}A/v.exec('AA')",
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
fn unicode_ids_binary_operator_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{IDSB}})+\B/d{mode}g,m=r.exec('😀\u2FF0\u2FF4\u31EF\uFEFF');m.index===2&&m[0]==='\u2FF0\u2FF4\u31EF'&&m.groups.x==='\u31EF'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}
