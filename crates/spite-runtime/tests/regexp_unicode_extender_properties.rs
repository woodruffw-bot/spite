//! Extender and its exact Ext alias over complete Unicode characters.
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
fn unicode_extender_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Extender}",
        r"\P{Ext}",
        r"[^\p{Ext}]",
        r"[\p{Ext}K]",
        r"[\P{Extender}K]",
        r"^(?<x>[\p{Ext}K])$",
        r"(?<x>\P{Extender})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\u{b6}\u{b7}\u{b8}\u{2cf}\u{2d0}\u{2d1}\u{2d2}\u{63f}\u{640}\u{641}\u{7f9}\u{7fa}\u{7fb}\u{a70}\u{a71}\u{a72}\u{afa}\u{afb}\u{afc}\u{b53}\u{b55}\u{b56}\u{e45}\u{e46}\u{e47}\u{ec5}\u{ec6}\u{ec7}\u{1809}\u{180a}\u{180b}\u{1842}\u{1843}\u{1844}\u{1aa6}\u{1aa7}\u{1aa8}\u{1c35}\u{1c36}\u{1c37}\u{1c7a}\u{1c7b}\u{1c7c}\u{3004}\u{3005}\u{3006}\u{3030}\u{3031}\u{3035}\u{3036}\u{309c}\u{309d}\u{309e}\u{309f}\u{30fb}\u{30fc}\u{30fe}\u{30ff}\u{a014}\u{a015}\u{a016}\u{a60b}\u{a60c}\u{a60d}\u{a9ce}\u{a9cf}\u{a9d0}\u{a9e5}\u{a9e6}\u{a9e7}\u{aa6f}\u{aa70}\u{aa71}\u{aadc}\u{aadd}\u{aade}\u{aaf2}\u{aaf3}\u{aaf4}\u{aaf5}\u{ff6f}\u{ff70}\u{ff71}\u{10780}\u{10781}\u{10782}\u{10783}\u{10d4d}\u{10d4e}\u{10d4f}\u{10d69}\u{10d6a}\u{10d6b}\u{10d6e}\u{10d6f}\u{10d70}\u{11236}\u{11237}\u{11238}\u{1135c}\u{1135d}\u{1135e}\u{113d1}\u{113d2}\u{113d3}\u{113d4}\u{115c5}\u{115c6}\u{115c8}\u{115c9}\u{11a97}\u{11a98}\u{11a99}\u{11dd8}\u{11dd9}\u{11dda}\u{16b41}\u{16b42}\u{16b43}\u{16b44}\u{16fdf}\u{16fe0}\u{16fe1}\u{16fe2}\u{16fe3}\u{16fe4}\u{16ff1}\u{16ff2}\u{16ff3}\u{16ff4}\u{1e13b}\u{1e13c}\u{1e13d}\u{1e13e}\u{1e5ee}\u{1e5ef}\u{1e5f0}\u{1e943}\u{1e944}\u{1e946}\u{1e947}\u{feff}ſK\n",
                "X\r\n\u{b7}\u{2028}\u{640}\u{2029}",
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
fn unicode_extender_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Extender}}/{mode},b=/\p{{Ext}}/i{mode},n=/\P{{Ext}}/i{mode};a.test('\u00B7')&&a.test('\u02D0')&&b.test('\u00B7')&&!a.test('\u200D')&&!b.test('\u005f')&&!b.test('ſ')&&!b.test('K')&&!n.test('\u00B7')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{Extender}}/{mode},s=/\s/{mode};p.test('\u00B7')&&p.test('\u02D0')&&!s.test('\u00B7')&&!s.test('\u02D0')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{Ext}}K]/i{mode},b=/[^\P{{Extender}}K]/i{mode},c=/[\p{{Ext}}\P{{Extender}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u00B7')&&!a.test('ſ')&&b.test('\u02D0')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Ext}}$/{mode},b=/^\P{{Ext}}$/{mode};a.test('\u00B7')&&!a.test('\u02D0\n')&&!a.test('')&&!a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_extender_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Ext}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Ext}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u00B7');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Ext}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u00B7');m.index===2&&m[0]==='\u00B7'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Ext}}/{mode}y;r.lastIndex=1;r.exec('😀\u00B7')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_extender_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'\u02D0😀\u00B7G'.matchAll(/(?<x>\P{{Ext}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'\u02D0\u00B7😀X'.replace(/(?<x>\p{{Extender}})/{mode}g,'<$1:$<x>>')==='<\u02D0:\u02D0><\u00B7:\u00B7>😀X'"
        ));
        check(&format!(
            r"'😀\u02D0\u00B7G'.split(/(\p{{Ext}})/{mode}).join('|')==='😀|\u02D0||\u00B7|G'&&'😀\u00B7'.search(/\p{{Ext}}/{mode})===2&&'\u02D0\u00B7😀'.match(/\p{{Ext}}/{mode}g).join('')==='\u02D0\u00B7'"
        ));
        check(&format!(
            r"let t='';let x='\u00B7\u02D0😀\u00B7'.replace(/(\p{{Ext}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_extender_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Extender})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Extender})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u00B7').groups.x==='\u00B7'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Ext}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Ext}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u00B7')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Ext}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u00B7')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_extender_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{Ext}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n\\u00B7'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='\\u00B7'&&m[3000]==='\\u00B7'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_extender_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Ext}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/[\p{Ext}&&A]/v.exec('A')",
        r"/[\p{Ext}--A]/v.exec('A')",
        r"/\p{Ext}+b/u.exec('AA')",
        r"/\p{Ext}A/v.exec('AA')",
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
fn unicode_extender_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/\B(?<x>\p{{Ext}})+\B/d{mode}g,m=r.exec('😀\u00B7\u02D0\u0640!');m.index===2&&m[0]==='\u00B7\u02D0\u0640'&&m.groups.x==='\u0640'&&m.indices.groups.x[0]===4&&m.indices.groups.x[1]===5&&r.lastIndex===5"
        ));
    }
}

#[test]
fn unicode_extender_property_unicode18_addition() {
    let mut comparisons = 0;
    for name in ["Extender", "Ext"] {
        for flags in ["u", "ui", "v", "vi"] {
            check(&format!(
                r"let p=new RegExp('^\\p{{{name}}}$','{flags}'),n=new RegExp('^\\P{{{name}}}$','{flags}');p.test('\u0B54')&&!n.test('\u0B54')"
            ));
            comparisons += 1;
        }
    }
    assert_eq!(comparisons, 8);
}
