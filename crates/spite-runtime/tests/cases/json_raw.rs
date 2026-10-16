//! JSON.rawJSON/JSON.isRawJSON primitive validation, frozen values, and branding.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn valid_primitive_text_is_preserved_without_number_rounding() {
    for text in [
        "null",
        "true",
        "false",
        "0",
        "-0",
        "1.00",
        "1E+02",
        "1e400",
        "-1e-400",
        "9007199254740993",
    ] {
        check(&format!(
            "let value=JSON.rawJSON('{text}');JSON.isRawJSON(value) && value.rawJSON==='{text}'"
        ));
    }
    check(
        "JSON.rawJSON(9007199254740993n).rawJSON==='9007199254740993' && JSON.rawJSON(-0).rawJSON==='0' && JSON.rawJSON(null).rawJSON==='null' && JSON.rawJSON(false).rawJSON==='false'",
    );
    check(
        "let text=JSON.rawJSON(9007199254740993n).rawJSON;JSON.parse(text,function(k,v,c){return BigInt(c.source);})===9007199254740993n",
    );
    check("let text='\"'+'a'.repeat(100000)+'\"';JSON.rawJSON(text).rawJSON===text");
    check(
        r#"JSON.rawJSON('"\\uD800"').rawJSON==='"\\uD800"' && JSON.rawJSON('"'+ '\uDC00' +'"').rawJSON==='"'+ '\uDC00' +'"' && JSON.rawJSON('"\\u0061"').rawJSON==='"\\u0061"'"#,
    );
}

#[test]
fn invalid_text_containers_and_surrounding_whitespace_throw_syntaxerror() {
    for text in [
        "",
        " ",
        "\t1",
        "1\n",
        "\rtrue",
        "false ",
        "{}",
        "[]",
        "{\"x\":1}",
        "[1]",
        "+1",
        "01",
        ".1",
        "1.",
        "1e",
        "1e+",
        "-",
        "--1",
        "True",
        "undefined",
        "NaN",
        "Infinity",
        "'x'",
        "/*x*/1",
        "1;",
        "\u{feff}1",
    ] {
        // Pass the text through an ordinary JS string escape, retaining its units.
        let encoded = format!("{text:?}");
        let source = format!("JSON.rawJSON({encoded})");
        assert!(
            matches!(
                Realm::default().eval(&source),
                Err(Error::Exception {
                    kind: ExceptionKind::SyntaxError,
                    ..
                })
            ),
            "{source}"
        );
    }
    check(
        "let C=SyntaxError;globalThis.SyntaxError=function(){throw 7;};let caught=false;try{JSON.rawJSON(' ');}catch(e){caught=e instanceof C && e.constructor===C;}caught",
    );
}

#[test]
fn conversion_is_ordered_once_and_symbol_or_abrupt_input_propagates() {
    check(
        "let trace='',text={[Symbol.toPrimitive](hint){trace+=hint;return '1.0';}};let result=JSON.rawJSON.call(Symbol(),text,(trace+='argument',7));result.rawJSON==='1.0' && trace==='argumentstring'",
    );
    check(
        "let marker={},caught=false;try{JSON.rawJSON({toString(){throw marker;}});}catch(e){caught=e===marker;}caught",
    );
    assert!(matches!(
        Realm::default().eval("JSON.rawJSON(Symbol())"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    check(
        "let count=0;JSON.rawJSON({toString(){count++;return '1';},valueOf(){throw 8;}}).rawJSON==='1' && count===1",
    );
}

#[test]
fn raw_objects_have_null_prototypes_and_exact_frozen_descriptors() {
    check(
        "let value=JSON.rawJSON('1'),d=Object.getOwnPropertyDescriptor(value,'rawJSON');Object.getPrototypeOf(value)===null && Reflect.ownKeys(value).join(',')==='rawJSON' && d.value==='1' && !d.writable && d.enumerable && !d.configurable && Object.isFrozen(value) && Object.isSealed(value) && !Object.isExtensible(value) && !Reflect.set(value,'rawJSON','2') && !Reflect.deleteProperty(value,'rawJSON') && !Reflect.defineProperty(value,'extra',{value:1}) && Reflect.setPrototypeOf(value,null) && !Reflect.setPrototypeOf(value,{}) && JSON.isRawJSON(value)",
    );
    check(
        "Object.defineProperty(Object.prototype,'rawJSON',{set(){throw 7;}});let raw=JSON.rawJSON('true');raw.rawJSON==='true' && Object.getPrototypeOf(raw)===null",
    );
}

#[test]
fn branding_never_reads_properties_or_coerces_and_cannot_be_inherited_or_copied() {
    check(
        "let raw=JSON.rawJSON('1'),fake={get rawJSON(){throw 7;},get [Symbol.toStringTag](){throw 8;},[Symbol.toPrimitive](){throw 9;}};JSON.isRawJSON(raw) && !JSON.isRawJSON(fake) && !JSON.isRawJSON(Object.create(raw)) && !JSON.isRawJSON({...raw}) && !JSON.isRawJSON(Object.assign({},raw)) && !JSON.isRawJSON([]) && !JSON.isRawJSON(()=>{}) && !JSON.isRawJSON(new Number(1)) && !JSON.isRawJSON(null) && !JSON.isRawJSON(undefined) && !JSON.isRawJSON('1') && !JSON.isRawJSON(1n) && !JSON.isRawJSON(Symbol())",
    );
    check("JSON.isRawJSON.call(Symbol(),JSON.rawJSON('1')) && !JSON.isRawJSON.call(null)");
}

#[test]
fn function_metadata_nonconstruction_and_collection_are_standard() {
    check(
        "JSON.rawJSON.name==='rawJSON' && JSON.rawJSON.length===1 && JSON.isRawJSON.name==='isRawJSON' && JSON.isRawJSON.length===1 && Object.getPrototypeOf(JSON.rawJSON)===Function.prototype && Object.getPrototypeOf(JSON.isRawJSON)===Function.prototype && !Object.hasOwn(JSON.rawJSON,'prototype') && !Object.hasOwn(JSON.isRawJSON,'prototype')",
    );
    for source in ["new JSON.rawJSON('1')", "new JSON.isRawJSON()"] {
        assert!(matches!(
            Realm::default().eval(source),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let raw=JSON.rawJSON,brand=JSON.isRawJSON,value=raw('1');delete JSON.rawJSON;delete JSON.isRawJSON;delete globalThis.JSON;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("brand(value) && brand(raw('2')) && value.rawJSON==='1'"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval("try{JSON.rawJSON('\"'+'a'.repeat(20000)+'\"');}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
