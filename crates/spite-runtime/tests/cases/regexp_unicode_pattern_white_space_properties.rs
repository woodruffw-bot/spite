//! Pattern_White_Space and its exact Pat_WS alias over complete Unicode characters.
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
fn unicode_pattern_white_space_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Pattern_White_Space}",
        r"\P{Pat_WS}",
        r"[^\p{Pat_WS}]",
        r"[\p{Pat_WS}K]",
        r"[\P{Pattern_White_Space}K]",
        r"^(?<x>[\p{Pat_WS}K])$",
        r"(?<x>\P{Pattern_White_Space})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{9}\u{a}\u{b}\u{c}\u{d}\u{20}\u{85}\u{200e}\u{200f}\u{2028}\u{2029}\u{8}\u{e}\u{1f}\u{21}\u{84}\u{86}\u{200d}\u{2010}\u{2027}\u{202a}\u{feff}ſK\n",
                "X\r\n\u{200e}\u{2028}\u{200f}\u{2029}",
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
fn unicode_pattern_white_space_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Pattern_White_Space}}/{mode},b=/\p{{Pat_WS}}/i{mode},n=/\P{{Pat_WS}}/i{mode};a.test('\u200E')&&a.test('\u200F')&&b.test('\u200E')&&!a.test('\u200D')&&!b.test('\u2010')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u200E')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Pattern_White_Space}}/{mode},s=/\s/{mode};p.test('\u200E')&&p.test('\u200F')&&!s.test('\u200E')&&!s.test('\u200F')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Pat_WS}}K]/i{mode},b=/[^\P{{Pattern_White_Space}}K]/i{mode},c=/[\p{{Pat_WS}}\P{{Pattern_White_Space}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u200E')&&!a.test('ſ')&&b.test('\u200F')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Pat_WS}}$/{mode},b=/^\P{{Pat_WS}}$/{mode};a.test('\u200E')&&!a.test('\u200F\n')&&!a.test('')&&a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_pattern_white_space_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Pat_WS}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Pat_WS}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u200e');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Pat_WS}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u200e');m.index===2&&m[0]==='\u200e'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Pat_WS}}/{mode}y;r.lastIndex=1;r.exec('😀\u200e')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_pattern_white_space_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u200f😀\u200eG'.matchAll(/(?<x>\P{{Pat_WS}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u200f\u200e😀X'.replace(/(?<x>\p{{Pattern_White_Space}})/{mode}g,'<$1:$<x>>')==='<\u200f:\u200f><\u200e:\u200e>😀X'"
        ));
        check(&format!(
            r"'😀\u200f\u200eG'.split(/(\p{{Pat_WS}})/{mode}).join('|')==='😀|\u200f||\u200e|G'&&'😀\u200e'.search(/\p{{Pat_WS}}/{mode})===2&&'\u200f\u200e😀'.match(/\p{{Pat_WS}}/{mode}g).join('')==='\u200f\u200e'"
        ));
        check(&format!(
            r"let t='';let x='\u200e\u200f😀\u200e'.replace(/(\p{{Pat_WS}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_pattern_white_space_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Pattern_White_Space})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Pattern_White_Space})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u200e').groups.x==='\u200e'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Pat_WS}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Pat_WS}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u200e')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Pat_WS}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u200e')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_pattern_white_space_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Pat_WS}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    check(r"/[\p{Pat_WS}&&A]/v.exec('A')===null");
    check(r"/[\p{Pat_WS}--A]/v.exec('A')===null");
    check(
        r"let s=String.fromCodePoint(0x200e),m=/[\p{Pat_WS}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [r"/\p{Pat_WS}+b/u.exec('AA')", r"/\p{Pat_WS}A/v.exec('AA')"] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
}

#[test]
fn unicode_pattern_white_space_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{Pat_WS}})+\B/d{mode}g,m=r.exec('😀\u200e\u200f\u200e!');m.index===2&&m[0]==='\u200e\u200f\u200e'&&m.groups.x==='\u200e'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}
