//! Distinct complete-character property and dot captures (22.2.2.3, 22.2.7.2).
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
fn unicode_property_dot_results_snapshot() {
    let mut rows = String::new();
    for body in [
        r"(\p{Script=Han})(.)",
        r"(\P{sc=Han})(.)",
        r"(\p{ASCII})(.)",
        r"(\P{ASCII})(.)",
        r"(\p{Join_C})(.)",
        r"(\p{Any})(.)",
        r"(?<p>\p{sc=Hani})(?<d>.)",
        r"(?<whole>((\p{sc=Han})(.)))",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "dvsy"] {
            for input in [
                JsString::from("𠮷a𠮷b𠮷"),
                JsString::from("一\n😀\u{200c}X"),
                JsString::from_code_units(vec![0xd800, 0x61, 0xdc00, 0xa, 0x62]),
            ] {
                for start in [0, 1, 2, input.len(), input.len() + 1] {
                    let source = format!(
                        "let r=new RegExp({pattern:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,groups:m.groups,indices:m.indices,indexGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
                    );
                    let Value::String(result) = Realm::default().eval(&source).unwrap() else {
                        panic!("{source}")
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
fn unicode_property_dot_distinct_names_and_global_captures() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(\p{{Script=Han}})(.)/d{mode}g,t='𠮷a𠮷b𠮷',m=r.exec(t);m[0]==='𠮷a'&&m[1]==='𠮷'&&m[2]==='a'&&m.index===0&&m.input===t&&m.indices[1][0]===0&&m.indices[1][1]===2&&m.indices[2][0]===2&&m.indices[2][1]===3&&r.lastIndex===3&&(m=r.exec(t),m[1]==='𠮷'&&m[2]==='b'&&m.index===3&&r.lastIndex===6)&&r.exec(t)===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let m=/(?<p>\p{{sc=Hani}})(?<__proto__>.)/d{mode}.exec('𠮷😀');m.groups.p==='𠮷'&&m.groups.__proto__==='😀'&&Object.getPrototypeOf(m.groups)===null&&m.indices.groups.p===m.indices[1]&&m.indices.groups.__proto__===m.indices[2]&&m.indices[1][1]===2&&m.indices[2][0]===2&&m.indices[2][1]===4"
        ));
        check(&format!(
            r"let m=/(?<whole>((\p{{sc=Han}})(.)))/d{mode}.exec('𠮷a');m.length===5&&m[1]==='𠮷a'&&m[2]==='𠮷a'&&m[3]==='𠮷'&&m[4]==='a'&&m.groups.whole===m[0]&&m.indices[3][1]===2&&m.indices[4][0]===2"
        ));
    }
}

#[test]
fn unicode_property_dot_pair_starts_and_failures() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let r=/(\p{{sc=Han}})(.)/d{mode}y;r.lastIndex=1;let m=r.exec('𠮷a');m.index===0&&m[1]==='𠮷'&&m[2]==='a'&&m.indices[1][0]===0&&m.indices[2][0]===2&&r.lastIndex===3"
        ));
        check(&format!(
            r"let r=/(\p{{sc=Han}})(.)/{mode}y;r.lastIndex=1;r.exec('😀𠮷a')===null&&r.lastIndex===0"
        ));
        check(&format!(
            r"let r=/(\p{{sc=Han}})(.)/{mode}g;r.lastIndex=1;let m=r.exec('😀𠮷a');m.index===2&&m[1]==='𠮷'&&m[2]==='a'&&r.lastIndex===5"
        ));
        check(&format!(
            r"let r=/(\p{{sc=Han}})(.)/{mode};r.lastIndex=77;let m=r.exec('𠮷a');m.index===0&&r.lastIndex===77"
        ));
    }
}

#[test]
fn unicode_property_dot_line_retries_and_folding() {
    for mode in ["u", "v"] {
        check(&format!(
            r"let a=/(\p{{sc=Han}})(.)/{mode}.exec('一\n𠮷a'),b=/(\p{{sc=Han}})(.)/s{mode}.exec('一\n𠮷a');a.index===2&&a[1]==='𠮷'&&a[2]==='a'&&b.index===0&&b[1]==='一'&&b[2]==='\n'"
        ));
        check(&format!(
            r"let m=/(\p{{ASCII}})(.)/i{mode}.exec('K😀');m[1]==='K'&&m[2]==='😀'&&m[0]==='K😀'&&m.index===0"
        ));
    }
    // Follow the existing tagged edition-17 folding rule; Node 24 differs here.
    check(
        r"let u=/(\P{ASCII})(.)/iu.exec('Ka'),v=/(\P{ASCII})(.)/iv.exec('Ka');u[1]==='K'&&u[2]==='a'&&v===null",
    );
}

#[test]
fn unicode_property_dot_consumers_original_text() {
    for mode in ["u", "v"] {
        check(&format!(
            r"'𠮷a𠮷b'.replace(/(\p{{sc=Han}})(.)/{mode}g,'<$1:$2>')==='<𠮷:a><𠮷:b>'"
        ));
        check(&format!(
            r"let a=[...'𠮷a𠮷b'.matchAll(/(?<p>\p{{sc=Han}})(?<d>.)/d{mode}g)];a.length===2&&a[0].groups.p==='𠮷'&&a[0].groups.d==='a'&&a[1].index===3&&a[1].indices[1][0]===3&&a[1].indices[2][0]===5"
        ));
        check(&format!(
            r"'𠮷a𠮷b'.split(/(\p{{sc=Han}})(.)/{mode}).join('|')==='|𠮷|a||𠮷|b|'"
        ));
        check(&format!(
            r"let t='',s='😀𠮷a𠮷b'.replace(/(\p{{sc=Han}})(.)/{mode}g,(m,a,b,p)=>{{t+=p;return a+':'+b}});s==='😀𠮷:a𠮷:b'&&t==='25'"
        ));
    }
}

#[test]
fn unicode_property_dot_clones_coercions_and_strict_writes() {
    check(
        r"let r=/(?<p>\p{sc=Han})(?<d>.)/dug,c=new RegExp(r,'dvsy');c.source===r.source&&c.unicodeSets&&!c.unicode&&c.dotAll&&c.exec('一\n').groups.d==='\n'&&c.lastIndex===2",
    );
    check(
        r"let t='',r=/(\p{sc=Han})(.)/uy;r.lastIndex={valueOf(){t+='i';return 1}};let m=r.exec({toString(){t+='s';return '𠮷a'}});t==='si'&&m[1]==='𠮷'&&m[2]==='a'&&m.index===0&&r.lastIndex===3",
    );
    check(
        r"let r=/(\p{sc=Han})(.)/vy;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('𠮷a')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===0",
    );
    check(
        r"let r=/(\p{sc=Han})(.)/vy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.exec('😀𠮷a')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
}

#[test]
fn unicode_property_dot_large_enclosing_captures_and_collection() {
    assert_eq!(Limits::default().max_steps, None);
    assert_eq!(Limits::default().max_heap_entries, None);
    let mut realm = Realm::default();
    realm.eval("let p='('.repeat(3000)+'(\\\\p{sc=Han})(.)'+')'.repeat(3000),r=new RegExp(p,'dvg'),c=new RegExp(r),text='😀'.repeat(100000)+'\\n𠮷a'").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=c.exec(text);m.length===3003&&m[1]==='𠮷a'&&m[3000]==='𠮷a'&&m[3001]==='𠮷'&&m[3002]==='a'&&m.indices[3001][0]===200001&&m.indices[3001][1]===200003&&m.indices[3002][0]===200003&&m.indices[3002][1]===200004&&c.lastIndex===200004"),Ok(Value::Boolean(true)));
    realm.eval("p=null;r=null;c=null;text=null;m=null").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn unicode_property_dot_opted_work_and_remaining_compositions() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(\\p{sc=Han})(.)/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r"/(\p{sc=Han})(a)/u.exec('𠮷a')",
        r"/(\p{sc=Han})(.)+/v.exec('𠮷a')",
        r"/(\p{sc=Han}|a)(.)/u.exec('𠮷a')",
        r"/(\p{sc=Han})(.)x/v.exec('𠮷ax')",
        r"/(\p{Script_Extensions=Latin})(.)/v.exec('Aa')",
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
