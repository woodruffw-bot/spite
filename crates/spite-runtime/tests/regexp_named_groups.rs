//! Native named captures retain slots, null prototypes, and indices aliases.

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
fn named_capture_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        ("(?<x>a)", "d", "qa"),
        ("(?<x>a)", "", "a"),
        ("(?<x>a)(b)(?<y>c)", "d", "qabc"),
        ("(?<outer>(?<x>a))", "d", "a"),
        ("(?<x>a)|(?<y>b)", "d", "b"),
        ("(?<x>a)|(?<x>b)", "d", "b"),
        ("(?<outer>(?<x>a)|(?<x>b))", "d", "b"),
        ("(?<x>a)|(?<x>b)", "d", "a"),
        ("(?<x>a)|(?<x>b)", "d", "c"),
        ("(?<x>)(?<y>a)", "d", "a"),
        ("(?<x>a)*", "d", ""),
        ("(?<x>a)+", "d", "aaa"),
        ("(?<x>a|b)+", "d", "abab"),
        ("(?<x>a|b)*", "d", ""),
        ("(?<outer>x(?<x>a|b)+)y", "d", "qxabby"),
        ("(?<outer>x(?<x>[^z]|b))+y", "d", "xaxby"),
        (
            "(?<prefix>x)(?<outer>(?<x>a|b)c)+(?<suffix>y)",
            "d",
            "xacbcy",
        ),
        (r"(?<\u0078>a)", "d", "a"),
        (r"(?<\u{10400}>a)", "d", "a"),
        ("(?<𐐀>a)(?<µ>b)", "d", "ab"),
        ("(?<a\u{200c}>a)", "d", "a"),
        ("(?<__proto__>a)(?<constructor>b)(?<toString>c)", "d", "abc"),
        ("(?<x>µ|Μ)(?<y>ſ|S)", "di", "ΜS"),
        (r"(?<x>\uD800)(?<y>\uDC00)", "d", "𐀀"),
        ("(?<x>.)", "ds", "\n"),
        ("^(?<x>a|b)$", "dm", "q\nb\n"),
        (r"\b(?<x>a|b)\b", "d", " b "),
        ("(?<x>a|b)", "dg", "qab"),
        ("(?<x>a|b)", "dy", "ba"),
        ("(?<x>a|b)", "dy", "qa"),
        ("(a)", "d", "a"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let text = JsString::from(text);
        let program = format!(
            "let r=new RegExp({source:?},{flags:?}),a=r.exec({text:?});JSON.stringify(a===null?null:{{matches:[...a],index:a.index,input:a.input,groups:a.groups,indices:a.indices,indicesGroups:a.indices===undefined?undefined:a.indices.groups,keys:a.groups===undefined?[]:Object.keys(a.groups),nullPrototype:a.groups===undefined?true:Object.getPrototypeOf(a.groups)===null,lastIndex:r.lastIndex}})"
        );
        let value = Realm::default().eval(&program).unwrap();
        let Value::String(value) = value else {
            panic!("expected JSON: {value:?}");
        };
        writeln!(rows, "{source:?} flags={flags:?} input={text:?} {value:?}").unwrap();
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn slots_aliases_key_order_and_descriptors_follow_original_names() {
    check(
        r"let a=/(?<x>a)/d.exec('a'),p=a.indices[1];a.groups.x='z';a.indices.groups.x[0]=7;a.indices[1]=[8,9];a[1]==='a'&&p[0]===7&&a.indices.groups.x===p&&a.indices[1]!==p&&delete a.groups.x&&Object.hasOwn(a,'1')",
    );
    check(
        r"let a=/(?<x>a)(b)(?<y>c)/d.exec('qabc'),p=Object.getOwnPropertyDescriptor(a.groups,'x');Object.getPrototypeOf(a.groups)===null&&Object.getPrototypeOf(a.indices.groups)===null&&a.groups.x==='a'&&a.groups.y==='c'&&a.indices.groups.x===a.indices[1]&&a.indices.groups.y===a.indices[3]&&Object.keys(a.groups).join(',')==='x,y'&&p.writable&&p.enumerable&&p.configurable&&p.value==='a'",
    );
    check(
        r"let a=/(?<x>a)|(?<x>b)/d.exec('b');a[1]===undefined&&a[2]==='b'&&a.groups.x==='b'&&a.indices.groups.x===a.indices[2]&&Object.keys(a.groups).join(',')==='x'",
    );
    check(
        r"let a=/(?<x>a)|(?<y>b)/d.exec('b');a.groups.x===undefined&&Object.hasOwn(a.groups,'x')&&a.indices.groups.x===undefined&&Object.hasOwn(a.indices.groups,'x')&&a.indices.groups.y===a.indices[2]",
    );
    check(
        r"let a=/(?<x>)(?<y>a)/d.exec('a'),b=/(?<x>a)*/d.exec('');a.groups.x===''&&a.indices.groups.x===a.indices[1]&&a.indices.groups.x[0]===0&&a.indices.groups.x[1]===0&&b.groups.x===undefined&&Object.hasOwn(b.groups,'x')&&b.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?<__proto__>a)(?<constructor>b)(?<toString>c)/d.exec('abc');a.groups.__proto__==='a'&&a.groups.constructor==='b'&&a.groups.toString==='c'&&Object.getPrototypeOf(a.groups)===null&&Object.keys(a.groups).join(',')==='__proto__,constructor,toString'&&a.indices.groups.__proto__===a.indices[1]",
    );
}

#[test]
fn consumers_replacements_and_copies_observe_live_named_values() {
    check(r"'qabc'.replace(/(?<x>a)(b)(?<y>c)/,'<$<y>,$<x>>')==='q<c,a>'");
    check(
        r"let seen;let s='qabc'.replace(/(?<x>a)(b)(?<y>c)/,(whole,x,b,y,index,input,groups)=>{seen=[whole,x,b,y,index,input,groups.x,groups.y,Object.getPrototypeOf(groups)===null];return 'z'});s==='qz'&&seen.join(',')==='abc,a,b,c,1,qabc,a,c,true'",
    );
    check(
        r"let r=/(?<x>a|b)/dg,a=[...'qab'.matchAll(r)];a.length===2&&a[0].groups.x==='a'&&a[1].groups.x==='b'&&a[1].indices.groups.x===a[1].indices[1]&&r.lastIndex===0",
    );
    check(
        r"let r=/(?<x>a|b)/dg;Object.defineProperty(r,'source',{get(){throw 7}});let copy=new RegExp(r);copy.source==='(?<x>a|b)'&&copy.exec('b').groups.x==='b'&&copy.lastIndex===1&&r.lastIndex===0",
    );
    check(
        r"'qabc'.split(/(?<x>a)(b)(?<y>c)/).join(',')==='q,a,b,c,'&&'qabc'.search(/(?<x>a)(b)(?<y>c)/)===1",
    );
}

#[test]
fn intrinsic_result_creation_does_not_call_prototype_setters() {
    check(
        r"let calls=0;Object.defineProperty(Object.prototype,'x',{set(){calls++},configurable:true});Object.defineProperty(Array.prototype,'1',{set(){calls++},configurable:true});let a=/(?<x>a)/d.exec('a');calls===0&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&Object.getPrototypeOf(a.groups)===null",
    );
}

#[test]
fn deep_named_copies_survive_collection_with_unlimited_defaults() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('('.repeat(100000)+'(?<x>a)'+')'.repeat(100000),'d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('a');a.length===100002&&a.groups.x==='a'&&a.indices.groups.x===a.indices[100001]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("a.groups.x==='a'&&a.indices.groups.x===a.indices[100001]&&a.indices.groups.x[1]===1&&Object.getPrototypeOf(a.groups)===null"),Ok(Value::Boolean(true)));
}

#[test]
fn unsupported_backreferences_modes_and_conditionally_captured_local_choices_remain_distinct() {
    for source in [
        r"/(?:(?:(?:(?<x>a)(?:\k<x>)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(?<x>a)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(?:x(?:(?<n>a)|(?<n>b))y){2}){2})|){2,3}/.test('xay')",
        "/(?<x>\\u{D800})/u.test('a')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    let mut realm = Realm::new(Limits {
        max_properties: Some(200),
        ..Limits::default()
    });
    realm
        .eval("let r=new RegExp('('.repeat(1000)+'(?<x>a)'+')'.repeat(1000),'g'),flag=0")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec('a')}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}
