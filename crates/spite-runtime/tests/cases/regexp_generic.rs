//! Observable generic RegExp operations without native matching (22.2.6).

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn flags_gets_follow_spec_order_without_coercing_truthy_values() {
    check(
        "let get=Object.getOwnPropertyDescriptor(RegExp.prototype,'flags').get,t='',poison={[Symbol.toPrimitive](){throw 7;}},r={get hasIndices(){t+='d';return poison;},get global(){t+='g';return Symbol();},get ignoreCase(){t+='i';return 1n;},get multiline(){t+='m';return [];},get dotAll(){t+='s';return 'x';},get unicode(){t+='u';return 1;},get unicodeSets(){t+='v';return true;},get sticky(){t+='y';return {};}};get.call(r)==='dgimsuvy' && t==='dgimsuvy'",
    );
    check(
        "let get=Object.getOwnPropertyDescriptor(RegExp.prototype,'flags').get;get.call({hasIndices:NaN,global:0,ignoreCase:0n,multiline:'',dotAll:null,unicode:undefined,unicodeSets:false,sticky:-0})==='' && get.call(Object.create(null))===''",
    );
    check(
        "let get=Object.getOwnPropertyDescriptor(RegExp.prototype,'flags').get,r={get hasIndices(){delete this.global;this.unicodeSets=true;return true;},global:true};get.call(r)==='dv'",
    );
    check(
        "let get=Object.getOwnPropertyDescriptor(RegExp.prototype,'flags').get,t='';try{get.call({get unicode(){t+='u';throw 7;},get unicodeSets(){t+='v';},get sticky(){t+='y';}});}catch(e){t+=e;}t==='u7'",
    );
    check(
        "RegExp.prototype.flags==='' && RegExp.prototype.source==='(?:)' && RegExp.prototype.toString()==='/(?:)/'",
    );
    for name in [
        "dotAll",
        "global",
        "hasIndices",
        "ignoreCase",
        "multiline",
        "source",
        "sticky",
        "unicode",
        "unicodeSets",
    ] {
        let mut realm = Realm::default();
        realm
            .eval(&format!(
                "let g=Object.getOwnPropertyDescriptor(RegExp.prototype,'{name}').get"
            ))
            .unwrap();
        for receiver in [
            "undefined",
            "null",
            "1",
            "'s'",
            "true",
            "1n",
            "Symbol()",
            "{}",
            "Object.create(RegExp.prototype)",
            "{source:'a',flags:'g'}",
        ] {
            assert!(
                matches!(
                    realm.eval(&format!("g.call({receiver})")),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{name} {receiver}"
            );
        }
        if name != "source" {
            assert_eq!(realm.eval("g.call(RegExp.prototype)"), Ok(Value::Undefined));
        }
    }
}

#[test]
fn tostring_orders_gets_and_string_hint_conversion_and_preserves_borrowed_source() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r"let t='',f=RegExp.prototype.toString,r={get source(){t+='a';return {[Symbol.toPrimitive](h){t+=h;return '\ud800/';}};},get flags(){t+='b';return {toString(){t+='c';return 'z';}};}};f.call(r)"),Ok(Value::String(JsString::from_code_units(vec![47,0xd800,47,47,122]))));
    assert_eq!(
        realm.eval("t"),
        Ok(Value::String(JsString::from("astringbc")))
    );
    check(
        "RegExp.prototype.toString.call({})==='/undefined/undefined' && RegExp.prototype.toString.call({source:7,flags:null})==='/7/null'",
    );
    check(
        "let t='';try{RegExp.prototype.toString.call({get source(){t+='s';return {toString(){t+='c';throw 7;}};},get flags(){t+='f';}});}catch(e){t+=e;}t==='sc7'",
    );
    for receiver in ["undefined", "null", "1", "'s'", "true", "1n", "Symbol()"] {
        assert!(
            matches!(
                Realm::default().eval(&format!("RegExp.prototype.toString.call({receiver})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{receiver}"
        );
    }
    let mut realm = Realm::new(Limits {
        max_string_units: Some(32),
        ..Limits::default()
    });
    assert_eq!(realm.eval("let f=RegExp.prototype.toString,m=0;try{f.call({source:'x'.repeat(20),get flags(){throw 7;}});}catch(e){m=e;}m===7"),Ok(Value::Boolean(true)));
    assert!(matches!(
        realm.eval(
            "try{f.call({source:'x'.repeat(20),flags:'y'.repeat(20)});}catch{m=1;}finally{m=2;}"
        ),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("m"), Ok(Value::Number(7.0)));
    assert_eq!(
        realm.eval("f.call({source:'x'.repeat(15),flags:'y'.repeat(15)}).length"),
        Ok(Value::Number(32.0))
    );
}

#[test]
fn test_delegates_to_exec_after_argument_conversion_and_validates_its_result() {
    check(
        "let t='',r={get exec(){t+='e';return function(s){t+='c';return this===r && arguments.length===1 && s==='x'?{}:null;};}},s={[Symbol.toPrimitive](h){t+=h;return 'x';}};RegExp.prototype.test.call(r,s)===true && t==='stringec'",
    );
    check(
        "RegExp.prototype.test.call({exec(s){return s==='undefined'?null:{};}},undefined)===false && RegExp.prototype.test.call({exec(){return []; }},'')===true",
    );
    check(
        "let t='';try{RegExp.prototype.test.call({get exec(){t+='e';throw 8;}},{toString(){t+='s';throw 7;}});}catch(e){t+=e;}t==='s7'",
    );
    for result in ["undefined", "true", "7", "'x'", "1n", "Symbol()"] {
        assert!(
            matches!(
                Realm::default().eval(&format!(
                    "RegExp.prototype.test.call({{exec(){{return {result};}}}},'')"
                )),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{result}"
        );
    }
    assert!(matches!(
        Realm::default().eval("RegExp.prototype.test.call({exec:0},'')"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    check(
        "let t='';try{RegExp.prototype.test.call(7,{toString(){t+='s';throw 9;}});}catch(e){t=e instanceof TypeError?'type':t;}t==='type'",
    );
}

#[test]
fn reentrant_generic_getters_and_exec_callbacks_use_existing_native_stack_guards() {
    for setup in [
        "let f=RegExp.prototype.toString,r={get source(){return f.call(r);},flags:''};",
        "let f=Object.getOwnPropertyDescriptor(RegExp.prototype,'flags').get,r={get global(){return f.call(r);}};",
        "let f=RegExp.prototype.test,r={exec(){return f.call(r,'');}};",
    ] {
        let mut realm = Realm::default();
        realm.eval(setup).unwrap();
        realm.eval("let m=0").unwrap();
        assert!(
            matches!(
                realm.eval("try{f.call(r,'');}catch{m=1;}finally{m=2;}"),
                Err(Error::Limit { .. })
            ),
            "{setup}"
        );
        assert_eq!(realm.eval("m"), Ok(Value::Number(0.0)));
        assert_eq!(realm.eval("7"), Ok(Value::Number(7.0)));
    }
}
