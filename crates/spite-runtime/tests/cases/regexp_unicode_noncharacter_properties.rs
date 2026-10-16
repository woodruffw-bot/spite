//! Noncharacter_Code_Point and its exact NChar alias over complete Unicode characters.
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
fn unicode_noncharacter_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Noncharacter_Code_Point}",
        r"\P{NChar}",
        r"[^\p{NChar}]",
        r"[\p{NChar}K]",
        r"[\P{Noncharacter_Code_Point}K]",
        r"^(?<x>[\p{NChar}K])$",
        r"(?<x>\P{Noncharacter_Code_Point})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{fdcf}\u{fdd0}\u{fdef}\u{fdf0}\u{fffe}\u{ffff}ſK\n",
                "X\r\n\u{1fffe}\u{2028}\u{10ffff}\u{2029}",
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
fn unicode_noncharacter_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Noncharacter_Code_Point}}/{mode},b=/\p{{NChar}}/i{mode},n=/\P{{NChar}}/i{mode};a.test('\uFDD0')&&a.test('\uFFFF')&&b.test('\uFDD0')&&!a.test('\uFDCF')&&!b.test('\uFDF0')&&!b.test('ſ')&&!b.test('K')&&!n.test('\uFDD0')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Noncharacter_Code_Point}}/{mode},s=/\s/{mode};p.test('\uFDD0')&&p.test('\uFFFF')&&!s.test('\uFDD0')&&!s.test('\uFFFF')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{NChar}}K]/i{mode},b=/[^\P{{Noncharacter_Code_Point}}K]/i{mode},c=/[\p{{NChar}}\P{{Noncharacter_Code_Point}}]/{mode};a.test('K')&&a.test('k')&&a.test('\uFDD0')&&!a.test('ſ')&&b.test('\uFFFF')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{NChar}}$/{mode},b=/^\P{{NChar}}$/{mode};a.test('\uFDD0')&&!a.test('\uFFFF\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_noncharacter_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let p=/\p{{NChar}}/i{mode},good=true;for(let plane=0;plane<17;plane++){{let top=plane*65536+65535;good=good&&p.test(String.fromCodePoint(top))&&p.test(String.fromCodePoint(top-1))&&!p.test(String.fromCodePoint(top-2))}}good"
        ));
        check(&format!(
            r"let r=/(?<x>\p{{Noncharacter_Code_Point}})/d{mode}y;r.lastIndex=1;let m=r.exec('\u{{10ffff}}');m.index===0&&m[0]==='\u{{10ffff}}'&&m.indices.groups.x[0]===0&&m.indices.groups.x[1]===2&&r.lastIndex===2"
        ));

        check(&format!(
            r"let r=/^(?<x>([\p{{NChar}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{NChar}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\ufdd0');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{NChar}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\ufdd0');m.index===2&&m[0]==='\ufdd0'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{NChar}}/{mode}y;r.lastIndex=1;r.exec('😀\ufdd0')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_noncharacter_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\uffff😀\ufdd0G'.matchAll(/(?<x>\P{{NChar}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\uffff\ufdd0😀X'.replace(/(?<x>\p{{Noncharacter_Code_Point}})/{mode}g,'<$1:$<x>>')==='<\uffff:\uffff><\ufdd0:\ufdd0>😀X'"
        ));
        check(&format!(
            r"'😀\uffff\ufdd0G'.split(/(\p{{NChar}})/{mode}).join('|')==='😀|\uffff||\ufdd0|G'&&'😀\ufdd0'.search(/\p{{NChar}}/{mode})===2&&'\uffff\ufdd0😀'.match(/\p{{NChar}}/{mode}g).join('')==='\uffff\ufdd0'"
        ));
        check(&format!(
            r"let t='';let x='\ufdd0\uffff😀\ufdd0'.replace(/(\p{{NChar}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_noncharacter_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Noncharacter_Code_Point})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Noncharacter_Code_Point})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\ufdd0').groups.x==='\ufdd0'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{NChar}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{NChar}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\ufdd0')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{NChar}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\ufdd0')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_noncharacter_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{NChar}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\ufdd0'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\ufdd0'&&m[3000]==='\\ufdd0'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_noncharacter_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{NChar}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    for source in [
        r"/[\p{NChar}&&A]/v.exec('A')",
        r"/[\p{NChar}--A]/v.exec('A')",
        r"/\p{NChar}+b/u.exec('AA')",
        r"/\p{NChar}A/v.exec('AA')",
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
