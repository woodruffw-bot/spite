//! Standard Error constructors, causes, generic stringification, and ErrorData.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

const NAMES: [&str; 7] = [
    "Error",
    "EvalError",
    "RangeError",
    "ReferenceError",
    "SyntaxError",
    "TypeError",
    "URIError",
];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn calls_and_construction_create_distinct_error_instances() {
    for name in NAMES {
        check(&format!(
            "let a={name}('detail'),b=new {name}('detail');a!==b && a instanceof {name} && a instanceof Error && b instanceof {name} && Error.isError(a) && Error.isError(b) && a.name==='{name}' && a.message==='detail' && a.toString()==='{name}: detail'"
        ));
        check(&format!(
            "let F={name}.bind({{}},'bound');let e=new F;e instanceof {name} && e.message==='bound' && {name}.call(null,'call').message==='call'"
        ));
        check(&format!(
            "{name}().toString()==='{name}' && {name}(undefined).message==='' && {name}(null).message==='null' && {name}(42n).message==='42'"
        ));
        check(&format!(
            "!Error.isError({name}.prototype) && !Error.isError({{__proto__:{name}.prototype}}) && ({{}}).toString.call({name}.prototype)==='[object Object]' && ({{}}).toString.call({name}())==='[object Error]'"
        ));
    }
    check(
        "!Error.isError() && !Error.isError(null) && !Error.isError(1) && !Error.isError('Error') && !Error.isError({toString:()=>{throw 1;}})",
    );
}

#[test]
fn constructors_and_instances_have_only_standard_properties() {
    let mut realm = Realm::default();
    let Value::Object(base_constructor) = realm.eval("Error").unwrap() else {
        panic!("Error");
    };
    let Value::Object(base_prototype) = realm.eval("Error.prototype").unwrap() else {
        panic!("Error prototype");
    };
    for name in NAMES {
        let Value::Object(constructor) = realm.eval(name).unwrap() else {
            panic!("constructor");
        };
        let constructor = realm.inspect_object(&constructor).unwrap();
        assert!(constructor.is_callable() && constructor.is_constructor());
        for (key, value) in [
            ("name", Value::String(JsString::from(name))),
            ("length", Value::Number(1.0)),
        ] {
            let descriptor = constructor
                .own_property(&JsString::from(key))
                .unwrap()
                .as_data()
                .unwrap();
            assert_eq!(descriptor.value, value);
            assert!(!descriptor.writable && !descriptor.enumerable && descriptor.configurable);
        }
        let descriptor = constructor
            .own_property(&JsString::from("prototype"))
            .unwrap()
            .as_data()
            .unwrap();
        assert!(!descriptor.writable && !descriptor.enumerable && !descriptor.configurable);
        let Value::Object(prototype) = &descriptor.value else {
            panic!("prototype");
        };
        if name != "Error" {
            assert_eq!(constructor.prototype(), Some(&base_constructor));
            assert_eq!(
                realm.inspect_object(prototype).unwrap().prototype(),
                Some(&base_prototype)
            );
        }
        let Value::Object(instance) = realm.eval(&format!("new {name}(undefined)")).unwrap() else {
            panic!("instance");
        };
        let instance = realm.inspect_object(&instance).unwrap();
        assert!(instance.is_error());
        for key in ["message", "name", "cause", "stack"] {
            assert!(
                instance.own_property(&JsString::from(key)).is_none(),
                "{key}"
            );
        }
        let Value::Object(instance) = realm
            .eval(&format!("{name}('detail',{{cause:undefined}})"))
            .unwrap()
        else {
            panic!("instance");
        };
        let instance = realm.inspect_object(&instance).unwrap();
        for (key, value) in [
            ("message", Value::String(JsString::from("detail"))),
            ("cause", Value::Undefined),
        ] {
            let descriptor = instance
                .own_property(&JsString::from(key))
                .unwrap()
                .as_data()
                .unwrap();
            assert_eq!(descriptor.value, value);
            assert!(descriptor.writable && !descriptor.enumerable && descriptor.configurable);
        }
    }
    check(
        "Error.isError.length===1 && Error.isError.name==='isError' && Error.prototype.toString.length===0 && Error.prototype.toString.name==='toString'",
    );
    for source in ["new Error.isError()", "new Error.prototype.toString()"] {
        assert!(matches!(
            realm.eval(source),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn causes_use_presence_without_coercing_values_or_primitive_options() {
    check(
        "let cause={};let e=TypeError('text',{__proto__:{cause:cause}});e.cause===cause && 'cause' in e && delete e.cause && !('cause' in e)",
    );
    check(
        "let unused={valueOf:()=>{throw 1;},toString:()=>{throw 2;}};let e=Error('text',{cause:unused});e.cause===unused",
    );
    for options in ["undefined", "null", "true", "1", "1n", "'text'", "{}"] {
        check(&format!("!('cause' in Error(undefined,{options}))"));
    }
    check(
        "let order='';let e=Error({toString:()=>{order+='m';return 'text';},valueOf:()=>{throw 1;}},{cause:3});e.message==='text' && e.cause===3 && order==='m'",
    );
    assert_eq!(
        Realm::default().eval("Error({toString:()=>{throw 4;}},{cause:3})"),
        Err(Error::Thrown(Value::Number(4.0)))
    );
}

#[test]
fn stringification_is_generic_and_preserves_utf16_and_empty_fields() {
    for (object, expected) in [
        ("{}", "Error"),
        ("{name:undefined,message:undefined}", "Error"),
        ("{name:'',message:'text'}", "text"),
        ("{name:'Kind',message:''}", "Kind"),
        ("{name:'',message:''}", ""),
        ("{name:null,message:null}", "null: null"),
        ("{name:4,message:true}", "4: true"),
    ] {
        assert_eq!(
            Realm::default().eval(&format!("Error.prototype.toString.call({object})")),
            Ok(Value::String(JsString::from(expected)))
        );
    }
    check("Error.prototype.toString.call({name:'\\uD800',message:'\\uDC00'})==='\\uD800: \\uDC00'");
    check(
        "Error.prototype.toString.call({name:'',message:{toString:()=> 'converted'}})==='converted'",
    );
    for value in ["undefined", "null", "true", "1", "1n", "'text'"] {
        assert!(matches!(
            Realm::default().eval(&format!("Error.prototype.toString.call({value})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn constructor_roots_and_cause_edges_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let saved=TypeError;let e=saved('outer',{cause:Error('inner')});delete globalThis.Error;delete globalThis.TypeError").unwrap();
    realm.collect(100_000).unwrap();
    assert_eq!(realm.eval("e.cause.message==='inner' && saved.isError(e) && saved('next').toString()==='TypeError: next'"),Ok(Value::Boolean(true)));
}
