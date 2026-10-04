//! ParseJSON materialization; serialization remains open.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn all_json_values_and_binary64_boundaries_materialize() {
    check(
        "JSON.parse('null')===null && JSON.parse('true')===true && JSON.parse('false')===false && Object.is(JSON.parse('-0'),-0) && JSON.parse('1e400')===Infinity && Object.is(JSON.parse('-1e-400'),-0) && JSON.parse('9007199254740993')===9007199254740992 && JSON.parse('5e-324')===Number.MIN_VALUE",
    );
    check(
        r#"let value=JSON.parse(' {"a":[1,true,null,"x"],"b":{}} ');Array.isArray(value.a) && value.a.length===4 && value.a[0]===1 && value.a[1]===true && value.a[2]===null && value.a[3]==='x' && Object.getPrototypeOf(value)===Object.prototype && Object.getPrototypeOf(value.a)===Array.prototype && Object.getPrototypeOf(value.b)===Object.prototype"#,
    );
    check(
        r#"JSON.parse('"\\uD800\\uDC00"')==='\uD800\uDC00' && JSON.parse('"\\uD800"')==='\uD800' && JSON.parse('"'+ '\uDC00' +'"')==='\uDC00' && JSON.parse('"\\u2028\\u2029"')==='\u2028\u2029'"#,
    );
}

#[test]
fn duplicates_proto_names_and_setters_obey_json_data_property_semantics() {
    check(
        r#"let value=JSON.parse('{"z":1,"__proto__":{"x":3},"2":7,"z":2,"1":8,"__proto__":4}');value.z===2 && value.__proto__===4 && Object.getPrototypeOf(value)===Object.prototype && Object.keys(value).join(',')==='1,2,z,__proto__' && Object.hasOwn(value,'__proto__')"#,
    );
    check(
        r#"Object.defineProperty(Object.prototype,'x',{set(){throw 7;}});Object.defineProperty(Array.prototype,'0',{set(){throw 8;}});let value=JSON.parse('{"x":1,"a":[2]}'),d=Object.getOwnPropertyDescriptor(value,'x'),e=Object.getOwnPropertyDescriptor(value.a,'0');value.x===1 && value.a[0]===2 && d.writable && d.enumerable && d.configurable && e.writable && e.enumerable && e.configurable"#,
    );
    check(
        r#"let parse=JSON.parse,get=Object.getPrototypeOf,op=Object.prototype,ap=Array.prototype;globalThis.Object=function(){throw 7;};globalThis.Array=function(){throw 8;};let value=parse('{"a":[]}');get(value)===op && get(value.a)===ap"#,
    );
}

#[test]
fn input_conversion_precedes_syntax_and_noncallable_revivers_are_ignored() {
    check(
        "let calls=0,text={[Symbol.toPrimitive](hint){calls++;if(hint!=='string')throw 7;return '1';}},reviver={get call(){throw 8;},toString(){throw 9;}};JSON.parse.call(Symbol(),text,reviver)===1 && calls===1 && JSON.parse(7,7)===7 && JSON.parse(7n,false)===7 && JSON.parse(null)===null",
    );
    check(
        "let marker={},caught=false;try{JSON.parse({toString(){throw marker;}});}catch(e){caught=e===marker;}caught",
    );
    for source in [
        "JSON.parse(Symbol())",
        "JSON.parse({toString(){return {};},valueOf(){return {};}})",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{source}"
        );
    }
    check(
        "let called=false,caught=false;try{JSON.parse('[1,]',()=>{called=true;});}catch(e){caught=e instanceof SyntaxError;}caught && !called",
    );
}

#[test]
fn invalid_json_throws_intrinsic_syntaxerror() {
    for text in ["undefined", "NaN", "01", "[1,]", "{a:1}", "'x'", "/*x*/1"] {
        assert!(
            matches!(
                Realm::default().eval(&format!("JSON.parse(\"{text}\")")),
                Err(Error::Exception {
                    kind: ExceptionKind::SyntaxError,
                    ..
                })
            ),
            "{text}"
        );
    }
    check(
        "let C=SyntaxError;globalThis.SyntaxError=function(){throw 7;};let caught=false;try{JSON.parse('x');}catch(e){caught=e instanceof C && e.constructor===C;}caught",
    );
}

#[test]
fn metadata_nonconstruction_and_intrinsic_retention_are_standard() {
    check(
        "Object.getPrototypeOf(JSON)===Object.prototype && Object.prototype.toString.call(JSON)==='[object JSON]' && !Object.hasOwn(JSON,'prototype') && JSON.parse.name==='parse' && JSON.parse.length===2 && !Object.hasOwn(JSON.parse,'prototype')",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(JSON,'parse'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length'),t=Object.getOwnPropertyDescriptor(JSON,Symbol.toStringTag),g=Object.getOwnPropertyDescriptor(globalThis,'JSON');d.writable && !d.enumerable && d.configurable && !n.writable && !n.enumerable && n.configurable && !l.writable && !l.enumerable && l.configurable && !t.writable && !t.enumerable && t.configurable && g.writable && !g.enumerable && g.configurable",
    );
    assert!(matches!(
        Realm::default().eval("new JSON.parse('1')"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm
        .eval("let parse=JSON.parse;delete JSON.parse;delete globalThis.JSON;")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("parse('1')===1 && typeof JSON==='undefined'"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn deep_default_values_and_opted_in_host_aborts_are_safe() {
    check(
        "let value=JSON.parse('['.repeat(10000)+'0'+']'.repeat(10000));for(let i=0;i<10000;i++){if(!Array.isArray(value) || value.length!==1)throw 7;value=value[0];}value===0",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval(
            "try{JSON.parse('['.repeat(1000)+'0'+']'.repeat(1000));}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_heap_entries: Some(300),
        ..Limits::default()
    });
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval("try{JSON.parse('['+'[],'.repeat(99)+'[]]');}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
