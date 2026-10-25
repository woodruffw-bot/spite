//! Bidi_Control and its exact Bidi_C alias over complete Unicode characters.
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
fn unicode_bidi_control_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Bidi_Control}",
        r"\P{Bidi_C}",
        r"[^\p{Bidi_C}]",
        r"[\p{Bidi_C}K]",
        r"[\P{Bidi_Control}K]",
        r"^(?<x>[\p{Bidi_C}K])$",
        r"(?<x>\P{Bidi_Control})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{61b}\u{61c}\u{61d}\u{200d}\u{200e}\u{200f}\u{2010}\u{2029}\u{202a}\u{202b}\u{202c}\u{202d}\u{202e}\u{202f}\u{2065}\u{2066}\u{2067}\u{2068}\u{2069}\u{206a}ſK\n",
                "X\r\n\u{200e}\u{2028}\u{2069}\u{2029}",
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
fn unicode_bidi_control_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Bidi_Control}}/{mode},b=/\p{{Bidi_C}}/i{mode},n=/\P{{Bidi_C}}/i{mode};a.test('\u200E')&&a.test('\u200F')&&b.test('\u200E')&&!a.test('\u200D')&&!b.test('\u2010')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u200E')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Bidi_Control}}/{mode},s=/\s/{mode};p.test('\u200E')&&p.test('\u200F')&&!s.test('\u200E')&&!s.test('\u200F')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Bidi_C}}K]/i{mode},b=/[^\P{{Bidi_Control}}K]/i{mode},c=/[\p{{Bidi_C}}\P{{Bidi_Control}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u200E')&&!a.test('ſ')&&b.test('\u200F')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Bidi_C}}$/{mode},b=/^\P{{Bidi_C}}$/{mode};a.test('\u200E')&&!a.test('\u200F\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_bidi_control_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Bidi_C}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Bidi_C}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u200e');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Bidi_C}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u200e');m.index===2&&m[0]==='\u200e'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Bidi_C}}/{mode}y;r.lastIndex=1;r.exec('😀\u200e')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_bidi_control_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u200f😀\u200eG'.matchAll(/(?<x>\P{{Bidi_C}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u200f\u200e😀X'.replace(/(?<x>\p{{Bidi_Control}})/{mode}g,'<$1:$<x>>')==='<\u200f:\u200f><\u200e:\u200e>😀X'"
        ));
        check(&format!(
            r"'😀\u200f\u200eG'.split(/(\p{{Bidi_C}})/{mode}).join('|')==='😀|\u200f||\u200e|G'&&'😀\u200e'.search(/\p{{Bidi_C}}/{mode})===2&&'\u200f\u200e😀'.match(/\p{{Bidi_C}}/{mode}g).join('')==='\u200f\u200e'"
        ));
        check(&format!(
            r"let t='';let x='\u200e\u200f😀\u200e'.replace(/(\p{{Bidi_C}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_bidi_control_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Bidi_Control})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Bidi_Control})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u200e').groups.x==='\u200e'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Bidi_C}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Bidi_C}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u200e')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Bidi_C}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u200e')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_bidi_control_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Bidi_C}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    check(r"/[\p{Bidi_C}&&A]/v.exec('A')===null");
    check(r"/[\p{Bidi_C}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x200e),m=/[\p{Bidi_C}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [r"/\p{Bidi_C}+b/u.exec('AA')", r"/\p{Bidi_C}A/v.exec('AA')"] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}

#[test]
fn unicode_bidi_control_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{Bidi_C}})+\B/d{mode}g,m=r.exec('😀\u061c\u202e\u2069!');m.index===2&&m[0]==='\u061c\u202e\u2069'&&m.groups.x==='\u2069'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}
