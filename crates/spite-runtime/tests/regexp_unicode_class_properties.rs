//! ASCII/Any property unions inside flat Unicode classes.
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
fn unicode_class_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"[\p{ASCII}]",
        r"[^\p{ASCII}]",
        r"[\P{ASCII}K]",
        r"[^\P{ASCII}K]",
        r"[\p{ASCII}\P{ASCII}]",
        r"[\P{Any}K]",
        r"[^\P{Any}K]",
        r"[\p{Any}]",
        r"[\P{ASCII}\w]",
        r"(?<x>^[\P{ASCII}K]$)",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in ["aSkſK😀\n", "\r\n\u{2028}\u{2029}", "A\nſ\n😀", ""] {
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
fn unicode_class_property_unions_inversion_and_complement_order() {
    check(
        r"let p=/[\p{ASCII}]/iv,n=/[\P{ASCII}]/iv;p.test('ſ')&&p.test('K')&&!n.test('ſ')&&!n.test('K')&&!n.test('S')&&n.test('😀')",
    );
    check(
        r"let r=/[\P{ASCII}K]/iv;r.test('K')&&r.test('k')&&r.test('K')&&!r.test('S')&&!r.test('s')&&!r.test('ſ')&&r.test('😀')",
    );
    check(
        r"let r=/[\P{ASCII}K]/iu;r.test('K')&&r.test('k')&&r.test('K')&&r.test('S')&&r.test('s')&&r.test('ſ')&&r.test('😀')",
    );
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/[\p{{ASCII}}\P{{ASCII}}]/i{mode};r.test('A')&&r.test('ſ')&&r.test('K')&&r.test('😀')&&r.test('\uD800')"
        ));
        check(&format!(
            r"let r=/[\P{{Any}}K]/i{mode};r.test('K')&&r.test('k')&&r.test('K')&&!r.test('S')&&!r.test('😀')&&!r.test('\uD800')"
        ));
        check(&format!(
            r"let r=/[\P{{ASCII}}\w]/i{mode};r.test('A')&&r.test('ſ')&&r.test('K')&&r.test('😀')&&!r.test('!')"
        ));
        check(&format!(
            r"let r=/[^\P{{Any}}]/{mode},n=/[^\p{{Any}}]/{mode};r.test('A')&&r.test('😀')&&r.test('\n')&&r.test('\uD800')&&!n.test('A')&&!n.test('😀')"
        ));
    }
}

#[test]
fn unicode_class_property_original_captures_names_context_and_widths() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>^[\P{{ASCII}}K]$)/dim{mode}g,m=r.exec('xx\r\nK\u2028A\u2029');m.index===4&&m[1]==='K'&&m.groups.x==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[1][1]===5&&r.lastIndex===5"
        ));
        check(&format!(
            r"let r=/(?<__proto__>([\p{{Any}}]))/d{mode}y;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m[1]==='😀'&&m[2]==='😀'&&m.indices[2][1]===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/((^[\p{{Any}}]$))/dm{mode},m=r.exec('xx\n😀\n');m.index===3&&m[1]==='😀'&&m[2]==='😀'&&m.indices[1][1]===5&&!/^[\p{{ASCII}}]$/{mode}.test('A\n')"
        ));
    }
}

#[test]
fn unicode_class_property_consumers_original_substrings_and_indices() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A😀\uD800'.matchAll(/(?<x>[\p{{Any}}])/d{mode}g)];a.length===3&&a[0][1]==='A'&&a[1][1]==='😀'&&a[1].index===1&&a[1].indices[1][1]===3&&a[2][1]==='\uD800'"
        ));
        check(&format!(
            r"'A😀\uD800'.replace(/(?<x>[\p{{Any}}])/{mode}g,'<$1:$<x>>')==='<A:A><😀:😀><\uD800:\uD800>'"
        ));
        check(&format!(
            r"'A😀B'.split(/([\P{{ASCII}}])/{mode}).join('|')==='A|😀|B'"
        ));
        check(&format!(
            r"let r=/[\P{{ASCII}}]/{mode};r.lastIndex=99;'A😀'.search(r)===1&&r.lastIndex===99"
        ));
    }
}

#[test]
fn unicode_class_property_metadata_mode_clones_coercions_and_writes() {
    check(
        r"let r=/(?<x>[\P{ASCII}K])/diug,c=new RegExp(r,'divy');c.source==='(?<x>[\\P{ASCII}K])'&&c.ignoreCase&&c.unicodeSets&&!c.unicode&&r.exec('S')[1]==='S'&&c.exec('S')===null&&c.exec('K').groups.x==='K'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/([\p{Any}])/uy;r.lastIndex={valueOf(){t+='i';return 0}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[1]==='😀'&&r.lastIndex===2",
    );
    check(
        r"let r=/[\p{Any}]/uy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/[\P{Any}]/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_class_property_repeated_sets_collection_and_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='('.repeat(3000)+'['+'\\\\p{ASCII}'.repeat(10000)+']'+')'.repeat(3000),r=new RegExp(p,'divg'),c=new RegExp(r),text='😀'.repeat(100000)+'A'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='A'&&m[3000]==='A'&&m.indices[3000][0]===200000&&m.indices[3000][1]===200001&&c.lastIndex===200001"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_class_property_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/[\\p{ASCII}]/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/[\p{Assigned}]/u.exec('A')",
        r"/[\p{Script=Han}]/v.exec('𠮷')",
        r"/[\p{RGI_Emoji}]/v.exec('😀')",
        r"/[\p{ASCII}&&K]/v.exec('K')",
        r"/[\p{ASCII}--K]/v.exec('A')",
        r"/[\p{ASCII}]+/v.exec('A')",
        r"/[\p{ASCII}]x/u.exec('Ax')",
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
fn original_unicode_class_property_complete_programs() {
    let mut rows = String::new();
    for source in [
        r"/([\p{ASCII}])/iu.exec('a')",
        r"/[\p{ASCII}]/u.exec('A')",
        r"/[\p{ASCII}]/u.exec('a')",
        r"/^[\p{ASCII}]$/u.exec('a')",
        r"/[\p{ASCII}]/iv.exec('a')",
        r"/[\p{ASCII}]/u.exec('a')",
        r"/[\p{ASCII}]/v.exec('a')",
    ] {
        let program = format!("JSON.stringify(eval({:?}))", JsString::from(source));
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("{program}")
        };
        writeln!(rows, "{:?} {result:?}", JsString::from(source)).unwrap();
    }
    insta::assert_snapshot!(rows);
}
