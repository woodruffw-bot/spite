//! Pattern_Syntax and its exact Pat_Syn alias over complete Unicode characters.
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
fn unicode_pattern_syntax_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Pattern_Syntax}",
        r"\P{Pat_Syn}",
        r"[^\p{Pat_Syn}]",
        r"[\p{Pat_Syn}K]",
        r"[\P{Pattern_Syntax}K]",
        r"^(?<x>[\p{Pat_Syn}K])$",
        r"(?<x>\P{Pattern_Syntax})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{21}\u{2f}\u{3a}\u{40}\u{5b}\u{5e}\u{60}\u{7b}\u{7e}\u{a1}\u{a7}\u{a9}\u{ab}\u{ac}\u{ae}\u{b0}\u{b1}\u{b6}\u{bb}\u{bf}\u{d7}\u{f7}\u{2010}\u{2027}\u{2030}\u{203e}\u{2041}\u{2053}\u{2055}\u{205e}\u{2190}\u{245f}\u{2500}\u{2775}\u{2794}\u{2bff}\u{2e00}\u{2e7f}\u{3001}\u{3003}\u{3008}\u{3020}\u{3030}\u{fd3e}\u{fd3f}\u{fe45}\u{fe46}\u{20}\u{30}\u{39}\u{41}\u{5a}\u{5f}\u{61}\u{7a}\u{7f}\u{a0}\u{a8}\u{aa}\u{ad}\u{af}\u{b2}\u{b5}\u{b7}\u{ba}\u{bc}\u{be}\u{c0}\u{d6}\u{d8}\u{f6}\u{f8}\u{200f}\u{2028}\u{202f}\u{203f}\u{2040}\u{2054}\u{205f}\u{218f}\u{2460}\u{24ff}\u{2776}\u{2793}\u{2c00}\u{2dff}\u{2e80}\u{3000}\u{3004}\u{3007}\u{3021}\u{302f}\u{3031}\u{fd3d}\u{fd40}\u{fe44}\u{fe47}\u{feff}ſK\n",
                "X\r\n\u{21}\u{2028}\u{2337}\u{2029}",
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
fn unicode_pattern_syntax_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Pattern_Syntax}}/{mode},b=/\p{{Pat_Syn}}/i{mode},n=/\P{{Pat_Syn}}/i{mode};a.test('\u0021')&&a.test('\u0022')&&b.test('\u0021')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u0021')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Pattern_Syntax}}/{mode},s=/\s/{mode};p.test('\u0021')&&p.test('\u0022')&&!s.test('\u0021')&&!s.test('\u0022')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Pat_Syn}}K]/i{mode},b=/[^\P{{Pattern_Syntax}}K]/i{mode},c=/[\p{{Pat_Syn}}\P{{Pattern_Syntax}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u0021')&&!a.test('ſ')&&b.test('\u0022')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Pat_Syn}}$/{mode},b=/^\P{{Pat_Syn}}$/{mode};a.test('\u0021')&&!a.test('\u0022\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_pattern_syntax_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Pat_Syn}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Pat_Syn}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u0021');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Pat_Syn}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u0021');m.index===2&&m[0]==='\u0021'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Pat_Syn}}/{mode}y;r.lastIndex=1;r.exec('😀\u0021')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_pattern_syntax_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u0022😀\u0021G'.matchAll(/(?<x>\P{{Pat_Syn}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u0022\u0021😀X'.replace(/(?<x>\p{{Pattern_Syntax}})/{mode}g,'<$1:$<x>>')==='<\u0022:\u0022><\u0021:\u0021>😀X'"
        ));
        check(&format!(
            r"'😀\u0022\u0021G'.split(/(\p{{Pat_Syn}})/{mode}).join('|')==='😀|\u0022||\u0021|G'&&'😀\u0021'.search(/\p{{Pat_Syn}}/{mode})===2&&'\u0022\u0021😀'.match(/\p{{Pat_Syn}}/{mode}g).join('')==='\u0022\u0021'"
        ));
        check(&format!(
            r"let t='';let x='\u0021\u0022😀\u0021'.replace(/(\p{{Pat_Syn}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_pattern_syntax_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Pattern_Syntax})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Pattern_Syntax})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u0021').groups.x==='\u0021'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Pat_Syn}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Pat_Syn}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u0021')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Pat_Syn}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u0021')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_pattern_syntax_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{Pat_Syn}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u0021'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u0021'&&m[3000]==='\\u0021'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_pattern_syntax_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Pat_Syn}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/\p{Script_Extensions=Latin}/u.exec('A')",
        r"/[\p{Pat_Syn}&&A]/v.exec('A')",
        r"/[\p{Pat_Syn}--A]/v.exec('A')",
        r"/\p{Pat_Syn}+b/u.exec('AA')",
        r"/\p{Pat_Syn}A/v.exec('AA')",
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
fn unicode_pattern_syntax_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{Pat_Syn}})+\B/d{mode}g,m=r.exec('😀\u0021\u0022\u2337\uFEFF');m.index===2&&m[0]==='\u0021\u0022\u2337'&&m.groups.x==='\u2337'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}
