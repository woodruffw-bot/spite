//! Hex_Digit and its exact Hex alias over complete Unicode characters.
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
fn unicode_hex_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{Hex_Digit}",
        r"\P{Hex}",
        r"[^\p{Hex}]",
        r"[\p{Hex}K]",
        r"[\P{Hex_Digit}K]",
        r"^(?<x>[\p{Hex}K])$",
        r"(?<x>\P{Hex_Digit})",
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
fn unicode_hex_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/\p{{Hex_Digit}}/{mode},b=/\p{{Hex}}/i{mode},n=/\P{{Hex}}/i{mode};a.test('0')&&a.test('9')&&a.test('A')&&a.test('F')&&a.test('a')&&a.test('f')&&b.test('F')&&!a.test('G')&&!b.test('ſ')&&!b.test('K')&&b.test('Ｆ')&&b.test('０')&&b.test('ｆ')&&!b.test('Ｇ')&&!n.test('A')&&n.test('😀')&&n.test('\uD800')&&n.test('\n')"
        ));
        check(&format!(
            r"let a=/[\p{{Hex}}K]/i{mode},b=/[^\P{{Hex_Digit}}K]/i{mode},c=/[\p{{Hex}}\P{{Hex_Digit}}]/{mode};a.test('K')&&a.test('k')&&a.test('F')&&!a.test('ſ')&&b.test('F')&&!b.test('K')&&c.test('😀')&&c.test('A')&&c.test('\uD800')"
        ));
        check(&format!(
            r"let a=/^\p{{Hex}}$/{mode},b=/^\P{{Hex}}$/{mode};a.test('A')&&!a.test('A\n')&&!a.test('')&&b.test('\n')&&b.test('😀')&&!b.test('AA')"
        ));
    }
}

#[test]
fn unicode_hex_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/^(?<x>([\p{{Hex}}K]))$/dim{mode}g,m=r.exec('x\r\nK\u2028');m.index===3&&m[1]==='K'&&m[2]==='K'&&m.indices.groups.x===m.indices[1]&&m.indices[2][1]===4&&r.lastIndex===4"
        ));
        check(&format!(
            r"let r=/(?<__proto__>\P{{Hex}})/d{mode}y;r.lastIndex=1;let m=r.exec('😀A');m.index===0&&m[1]==='😀'&&m.indices[1][1]===2&&r.lastIndex===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]"
        ));
        check(&format!(
            r"let r=/\p{{Hex}}/d{mode}g;r.lastIndex=1;let m=r.exec('😀A');m.index===2&&m[0]==='A'&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/\p{{Hex}}/{mode}y;r.lastIndex=1;r.exec('😀A')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_hex_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A😀fG'.matchAll(/(?<x>\P{{Hex}})/d{mode}g)];a.length===2&&a[0][1]==='😀'&&a[0].index===1&&a[0].indices[1][1]===3&&a[1][1]==='G'"
        ));
        check(&format!(
            r"'A😀fG'.replace(/(?<x>\p{{Hex}})/{mode}g,'<$1:$<x>>')==='<A:A>😀<f:f>G'"
        ));
        check(&format!(
            r"'😀A0G'.split(/(\p{{Hex}})/{mode}).join('|')==='😀|A||0|G'&&'😀F'.search(/\p{{Hex}}/{mode})===2&&'A😀f'.match(/\p{{Hex}}/{mode}g).join('')==='Af'"
        ));
        check(&format!(
            r"let t='';let x='A😀f'.replace(/(\p{{Hex}})/{mode}g,(m,c,p)=>{{t+=p;return c.toLowerCase()}});x==='a😀f'&&t==='03'"
        ));
    }
}

#[test]
fn unicode_hex_properties_clones_coercions_and_strict_writes() {
    check(
        r"let r=/^(?<x>\p{Hex_Digit})$/dmu,c=new RegExp(r,'divy');c.source==='^(?<x>\\p{Hex_Digit})$'&&c.unicodeSets&&!c.unicode&&!c.multiline&&c.exec('F').groups.x==='F'&&c.lastIndex===1",
    );
    check(
        r"let t='',r=/\P{Hex}/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[0]==='😀'&&m.index===0&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Hex}/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('A')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\p{Hex}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀A')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_hex_properties_repeated_sources_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='^'+'('.repeat(3000)+'['+'\\\\p{Hex}'.repeat(10000)+']'+')'.repeat(3000)+'$',r=new RegExp(p,'dmvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\nF'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='F'&&m[3000]==='F'&&m.indices[3000][0]===200001&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_hex_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Hex}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/[\p{Hex}&&A]/v.exec('A')",
        r"/[\p{Hex}--A]/v.exec('A')",
        r"/\p{Hex}+/u.exec('AA')",
        r"/\p{Hex}A/v.exec('AA')",
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
fn original_unicode_hex_property_complete_program() {
    let source = r"/\p{Hex_Digit}/u.exec('A')";
    let Value::String(result) = Realm::default()
        .eval(&format!(
            "JSON.stringify(eval({:?}))",
            JsString::from(source)
        ))
        .unwrap()
    else {
        panic!("{source}")
    };
    insta::assert_snapshot!(format!("{:?} {result:?}", JsString::from(source)));
}
