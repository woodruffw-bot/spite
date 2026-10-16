//! Assigned General_Category membership other than Cn over complete Unicode characters.
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
fn unicode_assigned_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Assigned}",
        r"\P{Assigned}",
        r"[^\p{Assigned}]",
        r"[\p{Assigned}K]",
        r"[\P{Assigned}K]",
        r"^(?<x>[\p{Assigned}K])$",
        r"(?<x>\P{Assigned})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{378}A\u{e000}\u{fdd0}\u{ffff}\u{f0000}\u{10ffff}ſK\n",
                "X\r\nA\u{2028}B\u{2029}",
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
fn unicode_assigned_properties_members_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let p=/^\p{{Assigned}}$/i{mode},n=/^\P{{Assigned}}$/i{mode};p.test('A')&&p.test('K')&&p.test('😀')&&p.test('\uD800')&&p.test('\uE000')&&p.test('\uFEFF')&&!p.test('\u0378')&&!p.test('\uFFFF')&&n.test('\u0378')&&n.test('\uFFFF')&&!n.test('A')&&!n.test('\uD800')"
        ));
        check(&format!(
            r"let p=/[\p{{Assigned}}K]/i{mode},n=/[^\P{{Assigned}}K]/i{mode},all=/[\p{{Assigned}}\P{{Assigned}}]/{mode};p.test('k')&&p.test('ſ')&&p.test('😀')&&!p.test('\u0378')&&n.test('😀')&&!n.test('K')&&all.test('\u0378')&&all.test('\uD800')"
        ));
        check(&format!(
            r"let p=/^\p{{Assigned}}$/{mode},n=/^\P{{Assigned}}$/{mode};p.test('\n')&&!p.test('')&&!p.test('AA')&&!p.test('😀\n')&&n.test('\uFFFF')&&!n.test('AA')"
        ));
        check(&format!(
            r"let p=/[\p{{Assigned}}]/i{mode},n=/[^\p{{Assigned}}]/i{mode};p.test('ſ')&&p.test('K')&&!p.test('\u0378')&&n.test('\u0378')&&!n.test('ſ')&&!n.test('K')"
        ));
    }
}

#[test]
fn unicode_assigned_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Assigned}}K]))$/dim{mode}g,m=r.exec('\u0378\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\p{{Assigned}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀A');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\P{{Assigned}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u0378');m.index===2&&m[0]==='\u0378'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\P{{Assigned}}/{mode}y;r.lastIndex=1;r.exec('😀\u0378')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_assigned_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A😀\u0378B'.matchAll(/(?<x>\P{{Assigned}})/d{mode}g)];a.length===1&&a[0][1]==='\u0378'&&a[0].index===3&&a[0].indices[1][1]===4"
        ));
        check(&format!(
            r"'AB\u0378'.replace(/(?<x>\p{{Assigned}})/{mode}g,'<$1:$<x>>')==='<A:A><B:B>\u0378'"
        ));
        check(&format!(
            r"'\u0378AB\u0378'.split(/(\p{{Assigned}})/{mode}).join('|')==='\u0378|A||B|\u0378'&&'\u0378😀'.search(/\p{{Assigned}}/{mode})===1&&'AB\u0378'.match(/\p{{Assigned}}/{mode}g).join('')==='AB'"
        ));
        check(&format!(
            r"let t='',x='AB\u0378A'.replace(/(\p{{Assigned}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__\u0378_'&&t==='013'"
        ));
    }
}

#[test]
fn unicode_assigned_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Assigned})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Assigned})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('A').groups.x==='A'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\p{Assigned}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Assigned}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('A')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\P{Assigned}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u0378')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_assigned_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{Assigned}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='\\u0378'.repeat(100000)+'\\nA'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='A'&&m[3000]==='A'&&m.indices[3000][0]===100001&&m.indices[3000][1]===100002&&c.lastIndex===100002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_assigned_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Assigned}/ug,marker=0,text='\\u0378'.repeat(10000);r.lastIndex=1")
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
    check(
        r"let m=/[\p{Assigned}&&A]/v.exec('A');m!==null&&m.length===1&&m[0]==='A'&&m.index===0&&m.input==='A'",
    );
    check(r"/[\p{Assigned}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0xe000),m=/[\p{Assigned}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [
        r"/\p{Assigned}+b/u.exec('AA')",
        r"/\p{Assigned}A/v.exec('AA')",
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
fn unicode_assigned_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{Assigned}})+\B/d{mode}g,m=r.exec('\u0378😀😀😀\u0378');m.index===1&&m[0]==='😀😀😀'&&m.groups.x==='😀'&&m.indices.groups.x[0]===5&&m.indices.groups.x[1]===7&&r.lastIndex===7"
        ));
    }
}

#[test]
fn unicode_assigned_property_lone_surrogates_private_use_and_noncharacters() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let p=/^\p{{Assigned}}$/{mode},n=/^\P{{Assigned}}$/{mode};p.test('\uD800')&&p.test('\uDC00')&&p.test('\uE000')&&p.test('\u{{F0000}}')&&!p.test('\uFDD0')&&!p.test('\u{{10FFFF}}')&&n.test('\uFDD0')&&n.test('\u{{10FFFF}}')"
        ));
    }
}

#[test]
fn unicode_assigned_property_previous_unsupported_contexts() {
    for (source, expected) in [
        (
            r"JSON.stringify((/([\p{Assigned}])/iu.exec('a')))",
            r#"["a","a"]"#,
        ),
        (
            r"JSON.stringify((/[\p{Assigned}]/iv.exec('a')))",
            r#"["a"]"#,
        ),
        (r"JSON.stringify((/[\p{Assigned}]/u.exec('A')))", r#"["A"]"#),
        (r"JSON.stringify((/[\p{Assigned}]/u.exec('a')))", r#"["a"]"#),
        (r"JSON.stringify((/[\p{Assigned}]/v.exec('a')))", r#"["a"]"#),
        (r"JSON.stringify((/\p{Assigned}/iv.exec('a')))", r#"["a"]"#),
        (r"JSON.stringify((/\p{Assigned}/u.exec('A')))", r#"["A"]"#),
        (r"JSON.stringify((/\p{Assigned}/u.exec('a')))", r#"["a"]"#),
        (
            r"JSON.stringify((/^(\p{Assigned})$/u.exec('A')))",
            r#"["A","A"]"#,
        ),
        (
            r"JSON.stringify((/^[\p{Assigned}]$/u.exec('a')))",
            r#"["a"]"#,
        ),
        (r"JSON.stringify((/^\p{Assigned}$/v.exec('a')))", r#"["a"]"#),
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::String(JsString::from(expected))),
            "{source}"
        );
    }
}

#[test]
fn unicode_assigned_property_unicode18_additions() {
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u0558')&&!n.test('\u0558')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u0558')&&!n.test('\u0558')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u0558')&&!n.test('\u0558')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u0558')&&!n.test('\u0558')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u058B')&&!n.test('\u058B')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u058B')&&!n.test('\u058B')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u058B')&&!n.test('\u058B')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u058B')&&!n.test('\u058B')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u058C')&&!n.test('\u058C')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u058C')&&!n.test('\u058C')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u058C')&&!n.test('\u058C')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u058C')&&!n.test('\u058C')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u05C8')&&!n.test('\u05C8')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u05C8')&&!n.test('\u05C8')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u05C8')&&!n.test('\u05C8')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u05C8')&&!n.test('\u05C8')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u05C9')&&!n.test('\u05C9')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u05C9')&&!n.test('\u05C9')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u05C9')&&!n.test('\u05C9')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u05C9')&&!n.test('\u05C9')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u0B53')&&!n.test('\u0B53')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u0B53')&&!n.test('\u0B53')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u0B53')&&!n.test('\u0B53')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u0B53')&&!n.test('\u0B53')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u0B54')&&!n.test('\u0B54')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u0B54')&&!n.test('\u0B54')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u0B54')&&!n.test('\u0B54')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u0B54')&&!n.test('\u0B54')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u1ADE')&&!n.test('\u1ADE')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u1ADE')&&!n.test('\u1ADE')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u1ADE')&&!n.test('\u1ADE')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u1ADE')&&!n.test('\u1ADE')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u1ADF')&&!n.test('\u1ADF')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u1ADF')&&!n.test('\u1ADF')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u1ADF')&&!n.test('\u1ADF')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u1ADF')&&!n.test('\u1ADF')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u1AEC')&&!n.test('\u1AEC')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u1AEC')&&!n.test('\u1AEC')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u1AEC')&&!n.test('\u1AEC')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u1AEC')&&!n.test('\u1AEC')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u1AEE')&&!n.test('\u1AEE')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u1AEE')&&!n.test('\u1AEE')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u1AEE')&&!n.test('\u1AEE')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u1AEE')&&!n.test('\u1AEE')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u1AF0')&&!n.test('\u1AF0')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u1AF0')&&!n.test('\u1AF0')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u1AF0')&&!n.test('\u1AF0')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u1AF0')&&!n.test('\u1AF0')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u208F')&&!n.test('\u208F')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u208F')&&!n.test('\u208F')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u208F')&&!n.test('\u208F')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u208F')&&!n.test('\u208F')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u209D')&&!n.test('\u209D')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u209D')&&!n.test('\u209D')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u209D')&&!n.test('\u209D')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u209D')&&!n.test('\u209D')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u209E')&&!n.test('\u209E')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u209E')&&!n.test('\u209E')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u209E')&&!n.test('\u209E')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u209E')&&!n.test('\u209E')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u209F')&&!n.test('\u209F')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u209F')&&!n.test('\u209F')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u209F')&&!n.test('\u209F')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u209F')&&!n.test('\u209F')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u20C2')&&!n.test('\u20C2')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u20C2')&&!n.test('\u20C2')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u20C2')&&!n.test('\u20C2')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u20C2')&&!n.test('\u20C2')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u20C3')&&!n.test('\u20C3')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u20C3')&&!n.test('\u20C3')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u20C3')&&!n.test('\u20C3')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u20C3')&&!n.test('\u20C3')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u20C4')&&!n.test('\u20C4')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u20C4')&&!n.test('\u20C4')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u20C4')&&!n.test('\u20C4')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u20C4')&&!n.test('\u20C4')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u2E60')&&!n.test('\u2E60')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u2E60')&&!n.test('\u2E60')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u2E60')&&!n.test('\u2E60')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u2E60')&&!n.test('\u2E60')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u2E61')&&!n.test('\u2E61')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u2E61')&&!n.test('\u2E61')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u2E61')&&!n.test('\u2E61')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u2E61')&&!n.test('\u2E61')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u2E63')&&!n.test('\u2E63')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u2E63')&&!n.test('\u2E63')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u2E63')&&!n.test('\u2E63')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u2E63')&&!n.test('\u2E63')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\uA7DD')&&!n.test('\uA7DD')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\uA7DD')&&!n.test('\uA7DD')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\uA7DD')&&!n.test('\uA7DD')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\uA7DD')&&!n.test('\uA7DD')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\uA7E2')&&!n.test('\uA7E2')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\uA7E2')&&!n.test('\uA7E2')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\uA7E2')&&!n.test('\uA7E2')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\uA7E2')&&!n.test('\uA7E2')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\uAB6C')&&!n.test('\uAB6C')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\uAB6C')&&!n.test('\uAB6C')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\uAB6C')&&!n.test('\uAB6C')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\uAB6C')&&!n.test('\uAB6C')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\uAB6D')&&!n.test('\uAB6D')");
    check(r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\uAB6D')&&!n.test('\uAB6D')");
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\uAB6D')&&!n.test('\uAB6D')");
    check(r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\uAB6D')&&!n.test('\uAB6D')");
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{107BB}')&&!n.test('\u{107BB}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{107BB}')&&!n.test('\u{107BB}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{107BB}')&&!n.test('\u{107BB}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{107BB}')&&!n.test('\u{107BB}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{107BD}')&&!n.test('\u{107BD}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{107BD}')&&!n.test('\u{107BD}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{107BD}')&&!n.test('\u{107BD}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{107BD}')&&!n.test('\u{107BD}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{107BF}')&&!n.test('\u{107BF}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{107BF}')&&!n.test('\u{107BF}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{107BF}')&&!n.test('\u{107BF}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{107BF}')&&!n.test('\u{107BF}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{10EC9}')&&!n.test('\u{10EC9}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{10EC9}')&&!n.test('\u{10EC9}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{10EC9}')&&!n.test('\u{10EC9}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{10EC9}')&&!n.test('\u{10EC9}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{10ECC}')&&!n.test('\u{10ECC}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{10ECC}')&&!n.test('\u{10ECC}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{10ECC}')&&!n.test('\u{10ECC}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{10ECC}')&&!n.test('\u{10ECC}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{10ECF}')&&!n.test('\u{10ECF}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{10ECF}')&&!n.test('\u{10ECF}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{10ECF}')&&!n.test('\u{10ECF}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{10ECF}')&&!n.test('\u{10ECF}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{10ED9}')&&!n.test('\u{10ED9}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{10ED9}')&&!n.test('\u{10ED9}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{10ED9}')&&!n.test('\u{10ED9}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{10ED9}')&&!n.test('\u{10ED9}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{10EE3}')&&!n.test('\u{10EE3}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{10EE3}')&&!n.test('\u{10EE3}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{10EE3}')&&!n.test('\u{10EE3}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{10EE3}')&&!n.test('\u{10EE3}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{10EEE}')&&!n.test('\u{10EEE}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{10EEE}')&&!n.test('\u{10EEE}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{10EEE}')&&!n.test('\u{10EEE}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{10EEE}')&&!n.test('\u{10EEE}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{10EF0}')&&!n.test('\u{10EF0}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{10EF0}')&&!n.test('\u{10EF0}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{10EF0}')&&!n.test('\u{10EF0}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{10EF0}')&&!n.test('\u{10EF0}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{10EF4}')&&!n.test('\u{10EF4}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{10EF4}')&&!n.test('\u{10EF4}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{10EF4}')&&!n.test('\u{10EF4}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{10EF4}')&&!n.test('\u{10EF4}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{10EF9}')&&!n.test('\u{10EF9}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{10EF9}')&&!n.test('\u{10EF9}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{10EF9}')&&!n.test('\u{10EF9}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{10EF9}')&&!n.test('\u{10EF9}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{11B0A}')&&!n.test('\u{11B0A}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{11B0A}')&&!n.test('\u{11B0A}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{11B0A}')&&!n.test('\u{11B0A}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{11B0A}')&&!n.test('\u{11B0A}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{11DF0}')&&!n.test('\u{11DF0}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{11DF0}')&&!n.test('\u{11DF0}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{11DF0}')&&!n.test('\u{11DF0}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{11DF0}')&&!n.test('\u{11DF0}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{11DF1}')&&!n.test('\u{11DF1}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{11DF1}')&&!n.test('\u{11DF1}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{11DF1}')&&!n.test('\u{11DF1}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{11DF1}')&&!n.test('\u{11DF1}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1246F}')&&!n.test('\u{1246F}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1246F}')&&!n.test('\u{1246F}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1246F}')&&!n.test('\u{1246F}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1246F}')&&!n.test('\u{1246F}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{12475}')&&!n.test('\u{12475}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{12475}')&&!n.test('\u{12475}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{12475}')&&!n.test('\u{12475}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{12475}')&&!n.test('\u{12475}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1247A}')&&!n.test('\u{1247A}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1247A}')&&!n.test('\u{1247A}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1247A}')&&!n.test('\u{1247A}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1247A}')&&!n.test('\u{1247A}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1247F}')&&!n.test('\u{1247F}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1247F}')&&!n.test('\u{1247F}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1247F}')&&!n.test('\u{1247F}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1247F}')&&!n.test('\u{1247F}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{12550}')&&!n.test('\u{12550}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{12550}')&&!n.test('\u{12550}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{12550}')&&!n.test('\u{12550}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{12550}')&&!n.test('\u{12550}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{125EB}')&&!n.test('\u{125EB}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{125EB}')&&!n.test('\u{125EB}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{125EB}')&&!n.test('\u{125EB}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{125EB}')&&!n.test('\u{125EB}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{12686}')&&!n.test('\u{12686}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{12686}')&&!n.test('\u{12686}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{12686}')&&!n.test('\u{12686}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{12686}')&&!n.test('\u{12686}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{18CD6}')&&!n.test('\u{18CD6}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{18CD6}')&&!n.test('\u{18CD6}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{18CD6}')&&!n.test('\u{18CD6}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{18CD6}')&&!n.test('\u{18CD6}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{18CD8}')&&!n.test('\u{18CD8}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{18CD8}')&&!n.test('\u{18CD8}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{18CD8}')&&!n.test('\u{18CD8}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{18CD8}')&&!n.test('\u{18CD8}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{18CDA}')&&!n.test('\u{18CDA}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{18CDA}')&&!n.test('\u{18CDA}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{18CDA}')&&!n.test('\u{18CDA}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{18CDA}')&&!n.test('\u{18CDA}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{18D1F}')&&!n.test('\u{18D1F}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{18D1F}')&&!n.test('\u{18D1F}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{18D1F}')&&!n.test('\u{18D1F}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{18D1F}')&&!n.test('\u{18D1F}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{18D20}')&&!n.test('\u{18D20}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{18D20}')&&!n.test('\u{18D20}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{18D20}')&&!n.test('\u{18D20}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{18D20}')&&!n.test('\u{18D20}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{18E00}')&&!n.test('\u{18E00}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{18E00}')&&!n.test('\u{18E00}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{18E00}')&&!n.test('\u{18E00}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{18E00}')&&!n.test('\u{18E00}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{18FC8}')&&!n.test('\u{18FC8}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{18FC8}')&&!n.test('\u{18FC8}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{18FC8}')&&!n.test('\u{18FC8}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{18FC8}')&&!n.test('\u{18FC8}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{19191}')&&!n.test('\u{19191}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{19191}')&&!n.test('\u{19191}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{19191}')&&!n.test('\u{19191}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{19191}')&&!n.test('\u{19191}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{191A0}')&&!n.test('\u{191A0}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{191A0}')&&!n.test('\u{191A0}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{191A0}')&&!n.test('\u{191A0}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{191A0}')&&!n.test('\u{191A0}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{191B9}')&&!n.test('\u{191B9}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{191B9}')&&!n.test('\u{191B9}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{191B9}')&&!n.test('\u{191B9}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{191B9}')&&!n.test('\u{191B9}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{191D2}')&&!n.test('\u{191D2}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{191D2}')&&!n.test('\u{191D2}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{191D2}')&&!n.test('\u{191D2}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{191D2}')&&!n.test('\u{191D2}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1B123}')&&!n.test('\u{1B123}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1B123}')&&!n.test('\u{1B123}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1B123}')&&!n.test('\u{1B123}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1B123}')&&!n.test('\u{1B123}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1B125}')&&!n.test('\u{1B125}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1B125}')&&!n.test('\u{1B125}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1B125}')&&!n.test('\u{1B125}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1B125}')&&!n.test('\u{1B125}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1B128}')&&!n.test('\u{1B128}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1B128}')&&!n.test('\u{1B128}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1B128}')&&!n.test('\u{1B128}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1B128}')&&!n.test('\u{1B128}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1B168}')&&!n.test('\u{1B168}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1B168}')&&!n.test('\u{1B168}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1B168}')&&!n.test('\u{1B168}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1B168}')&&!n.test('\u{1B168}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1CED2}')&&!n.test('\u{1CED2}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1CED2}')&&!n.test('\u{1CED2}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1CED2}')&&!n.test('\u{1CED2}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1CED2}')&&!n.test('\u{1CED2}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1CED3}')&&!n.test('\u{1CED3}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1CED3}')&&!n.test('\u{1CED3}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1CED3}')&&!n.test('\u{1CED3}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1CED3}')&&!n.test('\u{1CED3}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1CED4}')&&!n.test('\u{1CED4}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1CED4}')&&!n.test('\u{1CED4}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1CED4}')&&!n.test('\u{1CED4}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1CED4}')&&!n.test('\u{1CED4}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1CEDD}')&&!n.test('\u{1CEDD}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1CEDD}')&&!n.test('\u{1CEDD}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1CEDD}')&&!n.test('\u{1CEDD}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1CEDD}')&&!n.test('\u{1CEDD}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1CEDE}')&&!n.test('\u{1CEDE}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1CEDE}')&&!n.test('\u{1CEDE}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1CEDE}')&&!n.test('\u{1CEDE}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1CEDE}')&&!n.test('\u{1CEDE}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1CEDF}')&&!n.test('\u{1CEDF}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1CEDF}')&&!n.test('\u{1CEDF}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1CEDF}')&&!n.test('\u{1CEDF}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1CEDF}')&&!n.test('\u{1CEDF}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1CEF1}')&&!n.test('\u{1CEF1}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1CEF1}')&&!n.test('\u{1CEF1}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1CEF1}')&&!n.test('\u{1CEF1}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1CEF1}')&&!n.test('\u{1CEF1}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1CEF7}')&&!n.test('\u{1CEF7}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1CEF7}')&&!n.test('\u{1CEF7}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1CEF7}')&&!n.test('\u{1CEF7}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1CEF7}')&&!n.test('\u{1CEF7}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1CEFD}')&&!n.test('\u{1CEFD}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1CEFD}')&&!n.test('\u{1CEFD}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1CEFD}')&&!n.test('\u{1CEFD}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1CEFD}')&&!n.test('\u{1CEFD}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1D127}')&&!n.test('\u{1D127}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1D127}')&&!n.test('\u{1D127}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1D127}')&&!n.test('\u{1D127}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1D127}')&&!n.test('\u{1D127}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1D128}')&&!n.test('\u{1D128}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1D128}')&&!n.test('\u{1D128}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1D128}')&&!n.test('\u{1D128}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1D128}')&&!n.test('\u{1D128}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1D1EB}')&&!n.test('\u{1D1EB}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1D1EB}')&&!n.test('\u{1D1EB}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1D1EB}')&&!n.test('\u{1D1EB}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1D1EB}')&&!n.test('\u{1D1EB}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1D1F5}')&&!n.test('\u{1D1F5}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1D1F5}')&&!n.test('\u{1D1F5}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1D1F5}')&&!n.test('\u{1D1F5}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1D1F5}')&&!n.test('\u{1D1F5}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1D1FF}')&&!n.test('\u{1D1FF}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1D1FF}')&&!n.test('\u{1D1FF}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1D1FF}')&&!n.test('\u{1D1FF}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1D1FF}')&&!n.test('\u{1D1FF}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1D250}')&&!n.test('\u{1D250}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1D250}')&&!n.test('\u{1D250}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1D250}')&&!n.test('\u{1D250}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1D250}')&&!n.test('\u{1D250}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1D268}')&&!n.test('\u{1D268}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1D268}')&&!n.test('\u{1D268}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1D268}')&&!n.test('\u{1D268}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1D268}')&&!n.test('\u{1D268}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1D281}')&&!n.test('\u{1D281}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1D281}')&&!n.test('\u{1D281}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1D281}')&&!n.test('\u{1D281}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1D281}')&&!n.test('\u{1D281}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1D6A6}')&&!n.test('\u{1D6A6}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1D6A6}')&&!n.test('\u{1D6A6}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1D6A6}')&&!n.test('\u{1D6A6}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1D6A6}')&&!n.test('\u{1D6A6}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DB00}')&&!n.test('\u{1DB00}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DB00}')&&!n.test('\u{1DB00}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DB00}')&&!n.test('\u{1DB00}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DB00}')&&!n.test('\u{1DB00}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DB0E}')&&!n.test('\u{1DB0E}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DB0E}')&&!n.test('\u{1DB0E}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DB0E}')&&!n.test('\u{1DB0E}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DB0E}')&&!n.test('\u{1DB0E}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DB1C}')&&!n.test('\u{1DB1C}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DB1C}')&&!n.test('\u{1DB1C}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DB1C}')&&!n.test('\u{1DB1C}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DB1C}')&&!n.test('\u{1DB1C}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF1F}')&&!n.test('\u{1DF1F}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF1F}')&&!n.test('\u{1DF1F}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF1F}')&&!n.test('\u{1DF1F}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF1F}')&&!n.test('\u{1DF1F}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF21}')&&!n.test('\u{1DF21}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF21}')&&!n.test('\u{1DF21}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF21}')&&!n.test('\u{1DF21}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF21}')&&!n.test('\u{1DF21}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF24}')&&!n.test('\u{1DF24}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF24}')&&!n.test('\u{1DF24}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF24}')&&!n.test('\u{1DF24}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF24}')&&!n.test('\u{1DF24}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF2B}')&&!n.test('\u{1DF2B}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF2B}')&&!n.test('\u{1DF2B}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF2B}')&&!n.test('\u{1DF2B}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF2B}')&&!n.test('\u{1DF2B}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF40}')&&!n.test('\u{1DF40}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF40}')&&!n.test('\u{1DF40}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF40}')&&!n.test('\u{1DF40}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF40}')&&!n.test('\u{1DF40}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF48}')&&!n.test('\u{1DF48}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF48}')&&!n.test('\u{1DF48}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF48}')&&!n.test('\u{1DF48}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF48}')&&!n.test('\u{1DF48}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF4A}')&&!n.test('\u{1DF4A}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF4A}')&&!n.test('\u{1DF4A}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF4A}')&&!n.test('\u{1DF4A}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF4A}')&&!n.test('\u{1DF4A}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF4D}')&&!n.test('\u{1DF4D}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF4D}')&&!n.test('\u{1DF4D}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF4D}')&&!n.test('\u{1DF4D}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF4D}')&&!n.test('\u{1DF4D}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF51}')&&!n.test('\u{1DF51}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF51}')&&!n.test('\u{1DF51}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF51}')&&!n.test('\u{1DF51}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF51}')&&!n.test('\u{1DF51}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF56}')&&!n.test('\u{1DF56}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF56}')&&!n.test('\u{1DF56}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF56}')&&!n.test('\u{1DF56}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF56}')&&!n.test('\u{1DF56}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF68}')&&!n.test('\u{1DF68}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF68}')&&!n.test('\u{1DF68}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF68}')&&!n.test('\u{1DF68}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF68}')&&!n.test('\u{1DF68}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF6A}')&&!n.test('\u{1DF6A}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF6A}')&&!n.test('\u{1DF6A}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF6A}')&&!n.test('\u{1DF6A}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF6A}')&&!n.test('\u{1DF6A}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF6C}')&&!n.test('\u{1DF6C}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF6C}')&&!n.test('\u{1DF6C}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF6C}')&&!n.test('\u{1DF6C}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF6C}')&&!n.test('\u{1DF6C}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF6E}')&&!n.test('\u{1DF6E}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF6E}')&&!n.test('\u{1DF6E}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF6E}')&&!n.test('\u{1DF6E}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF6E}')&&!n.test('\u{1DF6E}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF72}')&&!n.test('\u{1DF72}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF72}')&&!n.test('\u{1DF72}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF72}')&&!n.test('\u{1DF72}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF72}')&&!n.test('\u{1DF72}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF74}')&&!n.test('\u{1DF74}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF74}')&&!n.test('\u{1DF74}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF74}')&&!n.test('\u{1DF74}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF74}')&&!n.test('\u{1DF74}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF76}')&&!n.test('\u{1DF76}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF76}')&&!n.test('\u{1DF76}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF76}')&&!n.test('\u{1DF76}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF76}')&&!n.test('\u{1DF76}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF78}')&&!n.test('\u{1DF78}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF78}')&&!n.test('\u{1DF78}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF78}')&&!n.test('\u{1DF78}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF78}')&&!n.test('\u{1DF78}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF7A}')&&!n.test('\u{1DF7A}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF7A}')&&!n.test('\u{1DF7A}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF7A}')&&!n.test('\u{1DF7A}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF7A}')&&!n.test('\u{1DF7A}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF7C}')&&!n.test('\u{1DF7C}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF7C}')&&!n.test('\u{1DF7C}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF7C}')&&!n.test('\u{1DF7C}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF7C}')&&!n.test('\u{1DF7C}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF7E}')&&!n.test('\u{1DF7E}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF7E}')&&!n.test('\u{1DF7E}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF7E}')&&!n.test('\u{1DF7E}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF7E}')&&!n.test('\u{1DF7E}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF81}')&&!n.test('\u{1DF81}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF81}')&&!n.test('\u{1DF81}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF81}')&&!n.test('\u{1DF81}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF81}')&&!n.test('\u{1DF81}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF90}')&&!n.test('\u{1DF90}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF90}')&&!n.test('\u{1DF90}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF90}')&&!n.test('\u{1DF90}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF90}')&&!n.test('\u{1DF90}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF93}')&&!n.test('\u{1DF93}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF93}')&&!n.test('\u{1DF93}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF93}')&&!n.test('\u{1DF93}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF93}')&&!n.test('\u{1DF93}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF95}')&&!n.test('\u{1DF95}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF95}')&&!n.test('\u{1DF95}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF95}')&&!n.test('\u{1DF95}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF95}')&&!n.test('\u{1DF95}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DF96}')&&!n.test('\u{1DF96}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DF96}')&&!n.test('\u{1DF96}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DF96}')&&!n.test('\u{1DF96}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DF96}')&&!n.test('\u{1DF96}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DFCD}')&&!n.test('\u{1DFCD}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DFCD}')&&!n.test('\u{1DFCD}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DFCD}')&&!n.test('\u{1DFCD}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DFCD}')&&!n.test('\u{1DFCD}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DFE6}')&&!n.test('\u{1DFE6}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DFE6}')&&!n.test('\u{1DFE6}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DFE6}')&&!n.test('\u{1DFE6}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DFE6}')&&!n.test('\u{1DFE6}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1DFFF}')&&!n.test('\u{1DFFF}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1DFFF}')&&!n.test('\u{1DFFF}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1DFFF}')&&!n.test('\u{1DFFF}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1DFFF}')&&!n.test('\u{1DFFF}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1F1AE}')&&!n.test('\u{1F1AE}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1F1AE}')&&!n.test('\u{1F1AE}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1F1AE}')&&!n.test('\u{1F1AE}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1F1AE}')&&!n.test('\u{1F1AE}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1F6D9}')&&!n.test('\u{1F6D9}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1F6D9}')&&!n.test('\u{1F6D9}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1F6D9}')&&!n.test('\u{1F6D9}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1F6D9}')&&!n.test('\u{1F6D9}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1F7DA}')&&!n.test('\u{1F7DA}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1F7DA}')&&!n.test('\u{1F7DA}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1F7DA}')&&!n.test('\u{1F7DA}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1F7DA}')&&!n.test('\u{1F7DA}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1F7DB}')&&!n.test('\u{1F7DB}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1F7DB}')&&!n.test('\u{1F7DB}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1F7DB}')&&!n.test('\u{1F7DB}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1F7DB}')&&!n.test('\u{1F7DB}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1F7F1}')&&!n.test('\u{1F7F1}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1F7F1}')&&!n.test('\u{1F7F1}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1F7F1}')&&!n.test('\u{1F7F1}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1F7F1}')&&!n.test('\u{1F7F1}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1F7F8}')&&!n.test('\u{1F7F8}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1F7F8}')&&!n.test('\u{1F7F8}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1F7F8}')&&!n.test('\u{1F7F8}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1F7F8}')&&!n.test('\u{1F7F8}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1F7FF}')&&!n.test('\u{1F7FF}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1F7FF}')&&!n.test('\u{1F7FF}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1F7FF}')&&!n.test('\u{1F7FF}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1F7FF}')&&!n.test('\u{1F7FF}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1FA8B}')&&!n.test('\u{1FA8B}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1FA8B}')&&!n.test('\u{1FA8B}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1FA8B}')&&!n.test('\u{1FA8B}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1FA8B}')&&!n.test('\u{1FA8B}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1FA8C}')&&!n.test('\u{1FA8C}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1FA8C}')&&!n.test('\u{1FA8C}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1FA8C}')&&!n.test('\u{1FA8C}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1FA8C}')&&!n.test('\u{1FA8C}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1FA8D}')&&!n.test('\u{1FA8D}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1FA8D}')&&!n.test('\u{1FA8D}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1FA8D}')&&!n.test('\u{1FA8D}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1FA8D}')&&!n.test('\u{1FA8D}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1FACC}')&&!n.test('\u{1FACC}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1FACC}')&&!n.test('\u{1FACC}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1FACC}')&&!n.test('\u{1FACC}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1FACC}')&&!n.test('\u{1FACC}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1FADD}')&&!n.test('\u{1FADD}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1FADD}')&&!n.test('\u{1FADD}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1FADD}')&&!n.test('\u{1FADD}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1FADD}')&&!n.test('\u{1FADD}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1FAEB}')&&!n.test('\u{1FAEB}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1FAEB}')&&!n.test('\u{1FAEB}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1FAEB}')&&!n.test('\u{1FAEB}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1FAEB}')&&!n.test('\u{1FAEB}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1FAF9}')&&!n.test('\u{1FAF9}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1FAF9}')&&!n.test('\u{1FAF9}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1FAF9}')&&!n.test('\u{1FAF9}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1FAF9}')&&!n.test('\u{1FAF9}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{1FAFA}')&&!n.test('\u{1FAFA}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{1FAFA}')&&!n.test('\u{1FAFA}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{1FAFA}')&&!n.test('\u{1FAFA}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{1FAFA}')&&!n.test('\u{1FAFA}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{2B81E}')&&!n.test('\u{2B81E}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{2B81E}')&&!n.test('\u{2B81E}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{2B81E}')&&!n.test('\u{2B81E}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{2B81E}')&&!n.test('\u{2B81E}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{3D000}')&&!n.test('\u{3D000}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{3D000}')&&!n.test('\u{3D000}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{3D000}')&&!n.test('\u{3D000}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{3D000}')&&!n.test('\u{3D000}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{3E61F}')&&!n.test('\u{3E61F}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{3E61F}')&&!n.test('\u{3E61F}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{3E61F}')&&!n.test('\u{3E61F}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{3E61F}')&&!n.test('\u{3E61F}')",
    );
    check(r"let p=/^\p{Assigned}$/u,n=/^\P{Assigned}$/u;p.test('\u{3FC3F}')&&!n.test('\u{3FC3F}')");
    check(
        r"let p=/^\p{Assigned}$/ui,n=/^\P{Assigned}$/ui;p.test('\u{3FC3F}')&&!n.test('\u{3FC3F}')",
    );
    check(r"let p=/^\p{Assigned}$/v,n=/^\P{Assigned}$/v;p.test('\u{3FC3F}')&&!n.test('\u{3FC3F}')");
    check(
        r"let p=/^\p{Assigned}$/vi,n=/^\P{Assigned}$/vi;p.test('\u{3FC3F}')&&!n.test('\u{3FC3F}')",
    );
}
