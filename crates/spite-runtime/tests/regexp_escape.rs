//! String-only RegExp.escape and its intrinsic metadata (22.2.5.1).

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

#[test]
fn string_only_inputs_do_not_coerce_wrappers_or_poisoned_objects_and_ignore_this() {
    let mut realm = Realm::default();
    realm.eval("let touched=0,poison={get toString(){touched++;throw 7;},get valueOf(){touched++;throw 8;},get [Symbol.toPrimitive](){touched++;throw 9;}}").unwrap();
    for value in [
        "undefined",
        "null",
        "false",
        "0",
        "1n",
        "Symbol()",
        "[]",
        "{}",
        "new String('a')",
        "poison",
    ] {
        assert!(
            matches!(
                realm.eval(&format!("RegExp.escape({value})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{value}"
        );
    }
    assert_eq!(realm.eval("touched"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("RegExp.escape.call(poison,'a-💩',touched++)"),
        Ok(Value::String(JsString::from("\\x61\\x2d💩")))
    );
    assert_eq!(realm.eval("touched"), Ok(Value::Number(1.0)));
    assert_eq!(
        realm.eval("RegExp.escape('')"),
        Ok(Value::String(JsString::from("")))
    );
    assert_eq!(
        realm.eval(r"RegExp.escape('\ud800\ud800\udc00\udfff')"),
        Ok(Value::String(JsString::from("\\ud800𐀀\\udfff")))
    );
}

#[test]
fn constructor_and_prototype_metadata_are_reflectable_and_intrinsic_functions_stay_rooted() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("typeof RegExp==='function' && RegExp.name==='RegExp' && RegExp.length===2 && Object.getPrototypeOf(RegExp)===Function.prototype && Object.getPrototypeOf(RegExp.prototype)===Object.prototype && RegExp.prototype.constructor===RegExp && Object.prototype.toString.call(RegExp.prototype)==='[object Object]' && !Object.hasOwn(RegExp.prototype,'lastIndex')"), Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let d=Object.getOwnPropertyDescriptor(RegExp,'escape');d.value===RegExp.escape && d.writable && !d.enumerable && d.configurable && RegExp.escape.length===1 && RegExp.escape.name==='escape' && !Object.hasOwn(RegExp.escape,'prototype')"), Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let p=Object.getOwnPropertyDescriptor(RegExp,'prototype');p.value===RegExp.prototype && !p.writable && !p.enumerable && !p.configurable;"), Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("let s=Object.getOwnPropertyDescriptor(RegExp,Symbol.species);s.get.name==='get [Symbol.species]' && s.get.length===0 && s.set===undefined && !s.enumerable && s.configurable && s.get.call(7)===7 && RegExp[Symbol.species]===RegExp"), Ok(Value::Boolean(true)));
    for (key, name, length) in [
        ("'exec'", "exec", 1),
        ("'test'", "test", 1),
        ("'toString'", "toString", 0),
        ("Symbol.match", "[Symbol.match]", 1),
        ("Symbol.matchAll", "[Symbol.matchAll]", 1),
        ("Symbol.replace", "[Symbol.replace]", 2),
        ("Symbol.search", "[Symbol.search]", 1),
        ("Symbol.split", "[Symbol.split]", 2),
    ] {
        assert_eq!(realm.eval(&format!("var desc=Object.getOwnPropertyDescriptor(RegExp.prototype,{key});desc.value.name==='{name}' && desc.value.length==={length} && desc.writable && !desc.enumerable && desc.configurable && !Object.hasOwn(desc.value,'prototype')")),Ok(Value::Boolean(true)));
    }
    for name in [
        "dotAll",
        "flags",
        "global",
        "hasIndices",
        "ignoreCase",
        "multiline",
        "source",
        "sticky",
        "unicode",
        "unicodeSets",
    ] {
        assert_eq!(realm.eval(&format!("var desc=Object.getOwnPropertyDescriptor(RegExp.prototype,'{name}');desc.get.name==='get {name}' && desc.get.length===0 && desc.set===undefined && !desc.enumerable && desc.configurable && !Object.hasOwn(desc.get,'prototype')")),Ok(Value::Boolean(true)));
    }
    assert!(matches!(
        realm.eval("new RegExp.escape('a')"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    for source in [
        "RegExp('a')",
        "new RegExp('a')",
        "RegExp.prototype.exec.call({})",
    ] {
        assert!(
            matches!(realm.eval(source), Err(Error::Unsupported { .. })),
            "{source}"
        );
    }
    let Value::Object(escape) = realm.eval("RegExp.escape").unwrap() else {
        panic!("function")
    };
    realm.eval("delete globalThis.RegExp").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(&escape).unwrap().is_callable());
}

#[test]
fn exact_output_and_work_quotas_are_opt_in_and_host_aborts_skip_handlers() {
    assert_eq!(
        Realm::default().eval(r"RegExp.escape('\ud800'.repeat(20000)).length===120000"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_string_units: Some(6),
        ..Limits::default()
    });
    realm.eval("let f=RegExp.escape,m=0").unwrap();
    assert!(matches!(realm.eval("f('aaaa')"), Err(Error::Limit { .. })));
    for (source, expected) in [
        ("f('éééééé')", "éééééé"),
        (r"f('\ud800')", "\\ud800"),
        ("f('💩')", "💩"),
    ] {
        assert_eq!(
            realm.eval(source),
            Ok(Value::String(JsString::from(expected)))
        );
    }
    assert!(matches!(
        realm.eval(r"try{f('\ud800-');}catch{m=1;}finally{m=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(20000),
        ..Limits::default()
    });
    realm
        .eval("let f=RegExp.escape,m=0,s='_'.repeat(10000)")
        .unwrap();
    assert!(matches!(
        realm.eval("try{f(s);}catch{m=1;}finally{m=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
}
