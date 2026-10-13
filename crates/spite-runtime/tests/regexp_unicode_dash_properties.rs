//! Dash and its exact Dash alias over complete Unicode characters.
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
fn unicode_dash_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Dash}",
        r"\P{Dash}",
        r"[^\p{Dash}]",
        r"[\p{Dash}K]",
        r"[\P{Dash}K]",
        r"^(?<x>[\p{Dash}K])$",
        r"(?<x>\P{Dash})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{2d}\u{58a}\u{5be}\u{1400}\u{1806}\u{2010}\u{2011}\u{2012}\u{2013}\u{2014}\u{2015}\u{2053}\u{207b}\u{208b}\u{2212}\u{2e17}\u{2e1a}\u{2e3a}\u{2e3b}\u{2e40}\u{2e5d}\u{301c}\u{3030}\u{30a0}\u{fe31}\u{fe32}\u{fe58}\u{fe63}\u{ff0d}\u{10d6e}\u{10ead}\u{2c}\u{2e}\u{589}\u{58b}\u{5bd}\u{5bf}\u{13ff}\u{1401}\u{1805}\u{1807}\u{200f}\u{2016}\u{2052}\u{2054}\u{207a}\u{207c}\u{208a}\u{208c}\u{2211}\u{2213}\u{2e16}\u{2e18}\u{2e19}\u{2e1b}\u{2e39}\u{2e3c}\u{2e3f}\u{2e41}\u{2e5c}\u{2e5e}\u{301b}\u{301d}\u{302f}\u{3031}\u{309f}\u{30a1}\u{fe30}\u{fe33}\u{fe57}\u{fe59}\u{fe62}\u{fe64}\u{ff0c}\u{ff0e}\u{10d6d}\u{10d6f}\u{10eac}\u{10eae}\u{feff}ſK\n",
                "X\r\n\u{2010}\u{2028}\u{2015}\u{2029}",
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
fn unicode_dash_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Dash}}/{mode},b=/\p{{Dash}}/i{mode},n=/\P{{Dash}}/i{mode};a.test('\u2010')&&a.test('\u2011')&&b.test('\u2010')&&!a.test('\u200D')&&!b.test('\u2016')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u2010')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Dash}}/{mode},s=/\s/{mode};p.test('\u2010')&&p.test('\u2011')&&!s.test('\u2010')&&!s.test('\u2011')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Dash}}K]/i{mode},b=/[^\P{{Dash}}K]/i{mode},c=/[\p{{Dash}}\P{{Dash}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u2010')&&!a.test('ſ')&&b.test('\u2011')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Dash}}$/{mode},b=/^\P{{Dash}}$/{mode};a.test('\u2010')&&!a.test('\u2011\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_dash_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>\p{{Dash}})/d{mode}y;r.lastIndex=1;let m=r.exec('\uD803\uDD6E');m.index===0&&m[0]==='\uD803\uDD6E'&&m.groups.x===m[0]&&m.indices[1][0]===0&&m.indices[1][1]===2&&r.lastIndex===2"
        ));
    }
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Dash}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Dash}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u2010');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Dash}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u2010');m.index===2&&m[0]==='\u2010'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Dash}}/{mode}y;r.lastIndex=1;r.exec('😀\u2010')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_dash_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u2011😀\u2010G'.matchAll(/(?<x>\P{{Dash}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u2011\u2010😀X'.replace(/(?<x>\p{{Dash}})/{mode}g,'<$1:$<x>>')==='<\u2011:\u2011><\u2010:\u2010>😀X'"
        ));
        check(&format!(
            r"'😀\u2011\u2010G'.split(/(\p{{Dash}})/{mode}).join('|')==='😀|\u2011||\u2010|G'&&'😀\u2010'.search(/\p{{Dash}}/{mode})===2&&'\u2011\u2010😀'.match(/\p{{Dash}}/{mode}g).join('')==='\u2011\u2010'"
        ));
        check(&format!(
            r"let t='';let x='\u2010\u2011😀\u2010'.replace(/(\p{{Dash}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_dash_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Dash})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Dash})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u2010').groups.x==='\u2010'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Dash}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Dash}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u2010')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Dash}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u2010')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_dash_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{Dash}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u2010'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u2010'&&m[3000]==='\\u2010'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_dash_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Dash}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/[\p{Dash}&&A]/v.exec('A')",
        r"/[\p{Dash}--A]/v.exec('A')",
        r"/\p{Dash}+b/u.exec('AA')",
        r"/\p{Dash}A/v.exec('AA')",
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
fn unicode_dash_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{Dash}})+\B/d{mode}g,m=r.exec('😀\u2010\u2011\u2015!');m.index===2&&m[0]==='\u2010\u2011\u2015'&&m.groups.x==='\u2015'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}
