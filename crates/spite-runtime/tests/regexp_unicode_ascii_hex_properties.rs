//! ASCII_Hex_Digit and its exact AHex alias over complete Unicode characters.
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
fn unicode_ascii_hex_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{ASCII_Hex_Digit}",
        r"\P{AHex}",
        r"[^\p{AHex}]",
        r"[\p{AHex}K]",
        r"[\P{ASCII_Hex_Digit}K]",
        r"^(?<x>[\p{AHex}K])$",
        r"(?<x>\P{ASCII_Hex_Digit})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                "😀0FaGſK\n",
                "x\r\nF\u{2028}a\u{2029}",
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
fn unicode_ascii_hex_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{ASCII_Hex_Digit}}/{mode},b=/\p{{AHex}}/i{mode},n=/\P{{AHex}}/i{mode};a.test('0')&&a.test('9')&&a.test('A')&&a.test('F')&&a.test('a')&&a.test('f')&&b.test('F')&&!a.test('G')&&!b.test('ſ')&&!b.test('K')&&!b.test('Ｆ')&&!b.test('０')&&!n.test('A')&&n.test('😀')&&n.test('\uD800')&&n.test('\n')"
        ));
        check(&format!(
            r"let a=/[\p{{AHex}}K]/i{mode},b=/[^\P{{ASCII_Hex_Digit}}K]/i{mode},c=/[\p{{AHex}}\P{{ASCII_Hex_Digit}}]/{mode};a.test('K')&&a.test('k')&&a.test('F')&&!a.test('ſ')&&b.test('F')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{AHex}}$/{mode},b=/^\P{{AHex}}$/{mode};a.test('A')&&!a.test('A\n')&&!a.test('')&&b.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_ascii_hex_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{AHex}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{AHex}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀A');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{AHex}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀A');m.index===2&&m[0]==='A'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{AHex}}/{mode}y;r.lastIndex=1;r.exec('😀A')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_ascii_hex_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A😀fG'.matchAll(/(?<x>\P{{AHex}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'A😀fG'.replace(/(?<x>\p{{AHex}})/{mode}g,'<$1:$<x>>')==='<A:A>😀<f:f>G'"
        ));
        check(&format!(
            r"'😀A0G'.split(/(\p{{AHex}})/{mode}).join('|')==='😀|A||0|G'&&'😀F'.search(/\p{{AHex}}/{mode})===2&&'A😀f'.match(/\p{{AHex}}/{mode}g).join('')==='Af'"
        ));
        check(&format!(
            r"let t='';let x='A😀f'.replace(/(\p{{AHex}})/{mode}g,(m,c,p)=>{{t+=p;return c.toLowerCase()}});x==='a😀f'&&t==='03'"
        ));
    }
}

#[test]
fn unicode_ascii_hex_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{ASCII_Hex_Digit})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{ASCII_Hex_Digit})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('F').groups.x==='F'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{AHex}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{AHex}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('A')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{AHex}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀A')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_ascii_hex_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{AHex}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\nF'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='F'&&m[3000]==='F'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_ascii_hex_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{AHex}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/\p{Hex_Digit}/u.exec('A')",
        r"/[\p{AHex}&&A]/v.exec('A')",
        r"/[\p{AHex}--A]/v.exec('A')",
        r"/\p{AHex}+/u.exec('AA')",
        r"/\p{AHex}A/v.exec('AA')",
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
