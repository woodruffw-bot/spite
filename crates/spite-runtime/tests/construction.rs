//! EvaluateNew, ordinary base construction, and bound constructor forwarding.

mod common;
use common::REALM_ENTRIES;

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn truth(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn ordinary_constructors_initialize_fresh_instances_with_current_prototypes() {
    for source in [
        "function F(x){this.x=x;}let a=new F(7),b=new F(9);a!==b && a.x===7 && b.x===9 && a.constructor===F",
        "function F(){'use strict';this.x=7;}new F().x===7",
        "function F(){}F.prototype.marker=7;new F().marker===7",
        "function F(){}let p={marker:7};F.prototype=p;new F().marker===7",
        "function F(){}let p={marker:7};let o=new F(F.prototype=p);o.marker===7",
        "function F(){}F.prototype=null;new F().toString()==='[object Object]'",
        "function F(){}F.prototype=7;new F().valueOf().toString()==='[object Object]'",
        "function F(x=7){this.x=x;}new F().x===7",
        "function F(x=()=>this){this.get=x;}let o=new F;o.get()===o",
        "function F(a){arguments[0]=7;this.x=a;}new F(1).x===7",
        "function F(a){'use strict';arguments[0]=7;this.x=a;}new F(1).x===1",
    ] {
        truth(source);
    }
}

#[test]
fn base_construction_uses_object_returns_and_ignores_primitive_returns() {
    for value in ["undefined", "null", "false", "7", "1n", "'text'"] {
        truth(&format!(
            "function F(){{this.x=7;return {value};}}new F().x===7"
        ));
    }
    truth("let replacement={x:9};function F(){this.x=7;return replacement;}new F()===replacement");
    truth("function F(){return ()=>7;}new F()()===7");
    truth("function F(){return arguments;}new F(7)[0]===7");
    truth("function F(){try{return {x:1};}finally{return {x:7};}}new F().x===7");
    truth("function F(){this.x=7;try{return {x:1};}finally{return null;}}new F().x===7");
    truth("function F(){throw 7;}let result;try{new F();}catch(e){result=e;}result===7");
}

#[test]
fn constructor_lookup_and_arguments_precede_checks_and_allocation() {
    truth(
        "let order='';function F(a,b){order+='c';this.x=a+b;}let o=new ({[(order+='k','f')]:F})[(order+='r','f')](order+='a',order+='b');order==='krabc'",
    );
    for constructor in [
        "0",
        "null",
        "{}",
        "(()=>1)",
        "({}).toString",
        "(()=>1).bind(null)",
        "({}).toString.bind(null)",
    ] {
        truth(&format!(
            "let effect=0,caught=false;try{{new ({constructor})(effect=7);}}catch{{caught=true;}}caught && effect===7"
        ));
    }
    truth(
        "let effect=0,caught=false;try{new missing(effect=7);}catch{caught=true;}caught && effect===0",
    );
    truth(
        "let arrow=()=>1;arrow.prototype={};let caught=false;try{new arrow;}catch{caught=true;}caught",
    );
    truth("function F(){}let a=F;a.prototype={value:7};new a().value===7");
    assert!(matches!(
        Realm::default().eval("new 7"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn bound_construction_forwards_arguments_and_uses_the_target_prototype() {
    truth(
        "function F(a,b){this.sum=a+b;}let receiver={};let B=F.bind(receiver,3);let o=new B(4);o.sum===7 && receiver.sum===undefined && o.constructor===F",
    );
    truth(
        "function F(a,b,c){this.sum=a*100+b*10+c;}let B=F.bind(1,1).bind(2,2);new B(3).sum===123",
    );
    truth("function F(){this.ok=arguments.callee===F;}let B=F.bind(null);new B().ok");
    truth(
        "function F(){}let B=F.bind(null);B.prototype={marker:9};F.prototype={marker:7};new B().marker===7",
    );
    truth("function F(){return {marker:7};}let B=F.bind(null);new B().marker===7");
    truth("function F(a=()=>this){this.get=a;}let B=F.bind(null);let o=new B;o.get()===o");
}

#[test]
fn parsed_constructor_and_call_precedence_matches_execution() {
    truth("function F(){return function G(){this.x=7;};}new new F()().x===7");
    truth("function G(){this.x=7;}function F(){this.G=G;}new new F().G().x===7");
    truth("function F(){return ()=>7;}new F()()===7");
    truth("function F(x){this.x=x;}let o={F};new o.F(7).x===7");
    truth("function F(){return {next:()=>7};}new F().next()===7");
    truth("function F(){this.x=7;}function factory(){return F;}new (factory())().x===7");
}

#[test]
fn constructed_instances_and_escaped_receivers_survive_collection() {
    let mut realm = Realm::default();
    realm
        .eval("let saved;function F(){saved=()=>this;this.payload={x:7};}let instance=new F;")
        .unwrap();
    realm.collect(10000).unwrap();
    assert_eq!(
        realm.eval("saved()===instance && saved().payload.x===7"),
        Ok(Value::Boolean(true))
    );
    realm.eval("saved=null;instance=null;F=null").unwrap();
    assert_eq!(realm.collect(10000).unwrap().live, REALM_ENTRIES);
}

#[test]
fn construction_limits_abort_handlers_and_restore_call_state() {
    let mut realm = Realm::default();
    realm.eval("let flag=0;function F(){new F;}").unwrap();
    assert!(matches!(
        realm.eval("try{new F;}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("function G(){this.x=7;}new G().x"),
        Ok(Value::Number(7.0))
    );
    let mut realm = Realm::new(Limits {
        max_arguments: Some(3),
        ..Limits::default()
    });
    realm
        .eval("let flag=0;function F(){flag=9;}let B=F.bind(null,1,2);")
        .unwrap();
    assert!(matches!(
        realm.eval("try{new B(3,4);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert!(matches!(
        realm.eval("new F(1,2,3,flag=3)"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(3.0)));
}
