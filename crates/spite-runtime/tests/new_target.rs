//! NewTarget binding on calls/construction and lexical capture through arrows.

use spite_runtime::{Realm, Value};

fn truth(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn calls_bind_undefined_and_construction_binds_the_constructor() {
    for source in [
        "function F(){return new.target;}F()===undefined && F.call({})===undefined && new F()===F",
        "function F(){'use strict';return new.target;}F()===undefined && new F()===F",
        "function F(){this.target=new.target;}new F().target===F",
        "function F(a=new.target){this.target=a;}new F().target===F",
        "function F(a=new.target){return a;}F()===undefined && new F()===F",
        "function F(){return new.target;}let B=F.bind(null).bind({});B()===undefined && new B()===F",
        "function F(){this.name=new.target.name;}new F().name==='F'",
        "function F(){return delete new.target;}F()===true",
        "function F(){return ()=>new.target;}let a=F(),b=new F();a()===undefined && b()===F",
        "function F(a=()=>new.target){return a;}let a=F(),b=new F();a()===undefined && b()===F",
        "function F(){return (a=new.target)=>a;}new F()()===F",
        "function F(){return ()=>()=>new.target;}new F()()()===F",
        "function F(){return ()=>new.target;}new F().bind({})()===F",
    ] {
        truth(source);
    }
}

#[test]
fn nested_ordinary_functions_create_their_own_new_target_binding() {
    truth(
        "function F(){function G(){return new.target;}this.normal=G();this.constructed=new G();this.G=G;}let o=new F();o.normal===undefined && o.constructed===o.G",
    );
    truth("function F(){function G(){return ()=>new.target;}return G();}new F()()===undefined");
    truth(
        "function F(){function G(){return ()=>new.target;}this.get=new G();this.G=G;}let o=new F();o.get()===o.G",
    );
    truth(
        "function F(){let keep=()=>new.target;function G(){return keep();}return G();}new F()===F",
    );
}

#[test]
fn escaped_new_target_is_traced_even_without_a_constructor_prototype_edge() {
    let mut realm = Realm::default();
    realm.eval("let saved,expected;function F(){return ()=>new.target;}F.prototype=null;expected=F;saved=new F();F=null;").unwrap();
    realm.collect(10000).unwrap();
    assert_eq!(realm.eval("saved()===expected"), Ok(Value::Boolean(true)));
    realm.eval("expected=null").unwrap();
    realm.collect(10000).unwrap();
    assert_eq!(realm.eval("saved().name"), Ok(Value::String("F".into())));
    realm.eval("saved=null").unwrap();
    assert_eq!(realm.collect(10000).unwrap().live, 11);
}
