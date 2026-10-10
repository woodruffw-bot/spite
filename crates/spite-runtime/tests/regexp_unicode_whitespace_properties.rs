//! White_Space and its exact space alias over complete Unicode characters.
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
fn unicode_whitespace_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{White_Space}",
        r"\P{space}",
        r"[^\p{space}]",
        r"[\p{space}K]",
        r"[\P{White_Space}K]",
        r"^(?<x>[\p{space}K])$",
        r"(?<x>\P{White_Space})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀\t\u{85}\u{feff} \u{a0}X\n",
                "X\r\n \u{2028}\u{85}\u{2029}",
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
fn unicode_whitespace_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{White_Space}}/{mode},b=/\p{{space}}/i{mode},n=/\P{{space}}/i{mode};a.test(' ')&&a.test('\t')&&a.test('\u0085')&&a.test('\u00A0')&&a.test('\u3000')&&b.test('\u2028')&&!a.test('X')&&!b.test('\uFEFF')&&!b.test('\u180E')&&!b.test('\u200B')&&!n.test(' ')&&n.test('😀')&&n.test('\uD800')&&n.test('\uFEFF')"
        ));
        check(&format!(
            r"let p=/\p{{White_Space}}/{mode},s=/\s/{mode};p.test('\u0085')&&!s.test('\u0085')&&!p.test('\uFEFF')&&s.test('\uFEFF')"
        ));
        check(&format!(
            r"let a=/[\p{{space}}K]/i{mode},b=/[^\P{{White_Space}}K]/i{mode},c=/[\p{{space}}\P{{White_Space}}]/{mode};a.test('K')&&a.test('k')&&a.test('\u0085')&&!a.test('ſ')&&b.test(' ')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{space}}$/{mode},b=/^\P{{space}}$/{mode};a.test('\u0085')&&!a.test(' \n')&&!a.test('')&&a.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_whitespace_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{space}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{space}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀\u0085');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{space}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀\u0085');m.index===2&&m[0]==='\u0085'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{space}}/{mode}y;r.lastIndex=1;r.exec('😀\u0085')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_whitespace_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...' 😀\u0085G'.matchAll(/(?<x>\P{{space}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"' \u0085😀X'.replace(/(?<x>\p{{White_Space}})/{mode}g,'<$1:$<x>>')==='< : ><\u0085:\u0085>😀X'"
        ));
        check(&format!(
            r"'😀 \u0085G'.split(/(\p{{space}})/{mode}).join('|')==='😀| ||\u0085|G'&&'😀\u0085'.search(/\p{{space}}/{mode})===2&&' \u0085😀'.match(/\p{{space}}/{mode}g).join('')===' \u0085'"
        ));
        check(&format!(
            r"let t='';let x=' \t😀 '.replace(/(\p{{space}})/{mode}g,(m,c,p)=>{{t+=p;return '_'}});x==='__😀_'&&t==='014'"
        ));
    }
}

#[test]
fn unicode_whitespace_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{White_Space})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{White_Space})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('\u0085').groups.x==='\u0085'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{space}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{space}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('\u0085')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{space}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀\u0085')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_whitespace_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{space}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n '").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]===' '&&m[3000]===' '&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_whitespace_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{space}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/[\p{space}&&A]/v.exec('A')",
        r"/[\p{space}--A]/v.exec('A')",
        r"/\p{space}+/u.exec('AA')",
        r"/\p{space}A/v.exec('AA')",
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
