//! Mandatory empty captures throughout concatenations retain original spans.
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
fn unicode_positive_pure_empty_results_snapshot() {
    let mut rows = String::new();
    for pattern in [
        "((?:|))+()",
        "(()){2,4}(?:|)*()",
        "(?:(())(?:||)*()){3,}",
        "((?:|)())+?()",
        "(()(?:|)){2}()(()(?:)*?())+",
        "(?:|){3,9}()+(?:|)*(()()){1,3}?",
        "(?<first>(?:|))+(?<second>){2}(?:|){0,3}?",
        "(?<__proto__>(?:|)+?)+?()",
        "(?:(?<x>(?:|){7,9})()){2,4}?",
        "((?:|)?){1,3}()",
    ] {
        for flags in ["du", "dug", "duy", "diumy", "dvg", "disvmy"] {
            for input in [
                JsString::from(""),
                JsString::from("a😀b"),
                JsString::from("\r\n"),
                JsString::from_code_units(vec![0xd800, 0x61, 0xdc00, 0xd800, 0xdc00]),
            ] {
                for start in 0..=input.len() + 1 {
                    let p = JsString::from(pattern);
                    let script = format!(
                        "let r=new RegExp({p:?},'{flags}');r.lastIndex={start};let m=r.exec({input:?});JSON.stringify(m===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...m],index:m.index,input:m.input,indices:m.indices,groups:m.groups,indicesGroups:m.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
                    );
                    let Value::String(result) = Realm::default()
                        .eval(&script)
                        .unwrap_or_else(|e| panic!("{script}: {e:?}"))
                    else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{p:?} flags={flags:?} input={input:?} start={start} {result:?}"
                    )
                    .unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_positive_pure_empty_names_indices_and_consumers() {
    check(
        "let m=/(?<first>(?:|)*)+(?<__proto__>){3}/du.exec('😀');m.length===3&&m[1]===''&&m[2]===''&&m.groups.first===''&&m.groups.__proto__===''&&m.indices.groups.first===m.indices[1]&&m.indices.groups.__proto__===m.indices[2]&&m.indices[1]!==m.indices[2]&&Object.getPrototypeOf(m.groups)===null",
    );
    check(
        "let a=[...'😀a'.matchAll(/(?<x>(?:|)+?)+?()/dvg)];a.length===3&&a[0].index===0&&a[1].index===2&&a[2].index===3&&a[1][2]===''&&a[1].groups.x===''&&a[1].indices.groups.x===a[1].indices[1]",
    );
    check("JSON.stringify('😀a'.split(/((?:|){3,}){2,4}()/u))===JSON.stringify(['😀','','','a'])");
    check(
        "'😀a'.replace(/((?:||)*?)+()/ug,'<$1:$2>')==='<:>😀<:>a<:>'&&'😀a'.match(/((?:|){7,9})()/vg).length===3",
    );
    check("'😀a'.search(/((?:|)?){1,3}()/v)===0&&/((?:|)+)+()/u.test('')");
    check(
        "let r=/(?<x>(?:|)*)+()/dvy;r.lastIndex=1;let m=r.exec('😀');m.index===0&&m.indices[2][0]===0&&m.indices.groups.x===m.indices[1]&&r.lastIndex===0",
    );
}

#[test]
fn unicode_positive_pure_empty_coercion_and_strict_last_index() {
    check(
        "let log='',r=/((?:|){3,}){2,4}()/uy;r.lastIndex={valueOf(){log+='i';return 1}};let m=r.exec({toString(){log+='s';return '😀'}});log==='si'&&m.index===0&&m[1]===''&&m[2]===''&&r.lastIndex===0",
    );
    check(
        "let log='',r=/((?:|)+)+()/vi,i={valueOf(){log+='i';return 1}};r.lastIndex=i;let m=r.exec({toString(){log+='s';return '😀'}});log==='si'&&m[2]===''&&r.lastIndex===i",
    );
    check(
        "let r=/((?:|)*){3,}()/uy;r.lastIndex=1;Object.defineProperty(r,'lastIndex',{writable:false});let caught=false;try{r.test('😀')}catch(e){caught=e instanceof TypeError}caught&&r.lastIndex===1",
    );
    check(
        "let r=/((?:|)+)+()/v;r.lastIndex=99;Object.defineProperty(r,'lastIndex',{writable:false});r.test('😀')&&r.lastIndex===99",
    );
    check("let r=/((?:|)+)+()/uy;r.lastIndex=3;r.exec('😀')===null&&r.lastIndex===0");
}

#[test]
fn unicode_positive_pure_empty_explicit_work_abort_preserves_state() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/((?:|)*){3,}()/ug;r.lastIndex=1;let text='b'.repeat(20000)")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn unicode_positive_pure_empty_preserves_earlier_capture_results() {
    check(
        "let m=/(())()/du.exec('a');m.length===4&&m[1]===''&&m[2]===''&&m[3]===''&&m.indices[3][0]===0&&m.indices[3][1]===0",
    );
    check(
        "let m=/(?<x>)(?<y>(?:))/dvi.exec('😀');m.groups.x===''&&m.groups.y===''&&m.indices.groups.x===m.indices[1]&&m.indices.groups.y===m.indices[2]",
    );
    check(
        "let p='(?<x>((?:|)*))',r=new RegExp(p,'iv'),c=new RegExp(r,'dvy'),m=c.exec('');c.source===p&&c.flags==='dvy'&&m.groups.x===''&&m[2]===''&&m.indices.groups.x===m.indices[1]",
    );
}

#[test]
fn unicode_positive_pure_empty_former_whole_programs_snapshot() {
    let mut rows = String::new();
    for source in ["/()+/u.test('')", "/()+/u.exec('a')"] {
        let actual = Realm::default().eval(source).unwrap();
        assert!(
            matches!(actual, Value::Boolean(true) | Value::Object(_)),
            "{source}"
        );
        let Value::String(result) = Realm::default()
            .eval(&format!("JSON.stringify({source})"))
            .unwrap()
        else {
            panic!("{source}")
        };
        writeln!(rows, "{source:?} {result:?}").unwrap();
    }
    insta::assert_snapshot!(rows);
}
