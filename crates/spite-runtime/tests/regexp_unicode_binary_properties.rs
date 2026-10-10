//! Standalone ASCII/Any Unicode property escapes with original widths.
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
fn unicode_binary_property_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"\p{ASCII}",
        r"\P{ASCII}",
        r"\p{Any}",
        r"\P{Any}",
        r"(?<x>^\p{ASCII}$)",
        r"((^\P{ASCII}$))",
        r"\p{Any}$",
        r"^\p{Any}",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in ["aSkſK😀\n", "\r\n\u{2028}\u{2029}", "A\nſ\n😀", ""] {
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
fn unicode_binary_property_complement_order_and_complete_characters() {
    check(
        r"let p=/\p{ASCII}/iu,n=/\P{ASCII}/iu;p.test('ſ')&&p.test('K')&&n.test('S')&&n.test('s')&&n.test('K')&&n.test('k')&&!n.test('A')&&n.test('ſ')&&n.test('K')",
    );
    check(
        r"let p=/\p{ASCII}/iv,n=/\P{ASCII}/iv;p.test('ſ')&&p.test('K')&&!n.test('S')&&!n.test('s')&&!n.test('K')&&!n.test('k')&&!n.test('ſ')&&!n.test('K')&&n.test('😀')",
    );
    for mode in ["u", "v"] {
        check(&format!(
            r"let p=/\p{{ASCII}}/{mode},n=/\P{{ASCII}}/{mode};p.test('\u0000')&&p.test('\u007f')&&!p.test('\u0080')&&!p.test('ſ')&&n.test('\u0080')&&n.test('ſ')&&n.test('😀')"
        ));
        check(&format!(
            r"let p=/\p{{Any}}/d{mode}g,n=/\P{{Any}}/i{mode};let m=p.exec('😀');m[0]==='😀'&&m.indices[0][1]===2&&p.lastIndex===2&&!n.test('A')&&!n.test('😀')&&!n.test('\uD800')"
        ));
        check(&format!(
            r"let p=/\p{{Any}}/d{mode}y;p.lastIndex=1;let m=p.exec('😀');m.index===0&&m[0]==='😀'&&m.indices[0][0]===0&&m.indices[0][1]===2&&p.lastIndex===2"
        ));
        check(&format!(
            r"let p=/\p{{Any}}/d{mode};let m=p.exec('\uD800');m[0]==='\uD800'&&m.indices[0][1]===1&&p.test('\n')&&p.test('\r')&&p.test('\u2028')&&p.test('\u2029')"
        ));
    }
}

#[test]
fn unicode_binary_property_wrappers_context_names_and_aliases() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(?<x>^\p{{ASCII}}$)/dim{mode}g,m=r.exec('xx\r\nſ\u2028A\u2029');m.index===4&&m[1]==='ſ'&&m.groups.x==='ſ'&&m.indices.groups.x===m.indices[1]&&m.indices[1][1]===5&&r.lastIndex===5"
        ));
        check(&format!(
            r"let r=/(?<__proto__>(\P{{ASCII}}))/d{mode}y;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m[1]==='😀'&&m[2]==='😀'&&m.indices[2][1]===2&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.__proto__===m.indices[1]&&r.lastIndex===2"
        ));
        check(&format!(
            r"let r=/((^\p{{Any}}$))/dm{mode},m=r.exec('xx\n😀\n');m.index===3&&m[1]==='😀'&&m[2]==='😀'&&m.indices[1][1]===5&&!/\p{{Any}}$/{mode}.test('')&&!/^\p{{ASCII}}$/{mode}.test('A\n')"
        ));
    }
}

#[test]
fn unicode_binary_property_consumers_original_substrings_and_indices() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=[...'A😀\uD800'.matchAll(/(?<x>\p{{Any}})/d{mode}g)];a.length===3&&a[0][1]==='A'&&a[1][1]==='😀'&&a[1].index===1&&a[1].indices[1][1]===3&&a[2][1]==='\uD800'"
        ));
        check(&format!(
            r"'A😀\uD800'.replace(/(?<x>\p{{Any}})/{mode}g,'<$1:$<x>>')==='<A:A><😀:😀><\uD800:\uD800>'"
        ));
        check(&format!(
            r"'A😀B'.split(/(\P{{ASCII}})/{mode}).join('|')==='A|😀|B'"
        ));
        check(&format!(
            r"let r=/\P{{ASCII}}/{mode};r.lastIndex=99;'A😀'.search(r)===1&&r.lastIndex===99"
        ));
    }
}

#[test]
fn unicode_binary_property_metadata_mode_clones_coercions_and_writes() {
    check(
        r"let r=/(?<x>\P{ASCII})/diug,c=new RegExp(r,'divy');c.source==='(?<x>\\P{ASCII})'&&c.ignoreCase&&c.unicodeSets&&!c.unicode&&r.exec('S')[1]==='S'&&c.exec('S')===null&&c.exec('😀').groups.x==='😀'&&c.lastIndex===2",
    );
    check(
        r"let t='',r=/(\p{Any})/uy;r.lastIndex={valueOf(){t+='i';return 0}};let m=r.exec({toString(){t+='s';return '😀'}});t==='si'&&m[1]==='😀'&&r.lastIndex===2",
    );
    check(
        r"let r=/\p{Any}/uy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/\P{Any}/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_binary_property_collection_many_wrappers_and_defaults() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='('.repeat(3000)+'\\\\P{ASCII}'+')'.repeat(3000),r=new RegExp(p,'dug'),c=new RegExp(r),text='A'.repeat(200000)+'😀'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3001&&m[1]==='😀'&&m[3000]==='😀'&&m.indices[3000][0]===200000&&m.indices[3000][1]===200002&&c.lastIndex===200002"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_binary_property_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{ASCII}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/\p{Script=Han}/v.exec('𠮷')",
        r"/\p{RGI_Emoji}/v.exec('😀')",
        r"/[\p{Assigned}]/u.exec('A')",
        r"/\p{ASCII}+/v.exec('A')",
        r"/\p{ASCII}x/u.exec('Ax')",
        r"/\p{ASCII}|x/v.exec('A')",
        r"/^(\p{ASCII})$/u.exec('A')",
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
fn original_unicode_binary_property_complete_programs() {
    let mut rows = String::new();
    for source in [
        r"/^\p{ASCII}$/v.exec('a')",
        r"/\p{ASCII}/iv.exec('a')",
        r"/\p{ASCII}/u.exec('a')",
    ] {
        let program = format!("JSON.stringify(eval({:?}))", JsString::from(source));
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("{program}")
        };
        writeln!(rows, "{:?} {result:?}", JsString::from(source)).unwrap();
    }
    insta::assert_snapshot!(rows);
}
