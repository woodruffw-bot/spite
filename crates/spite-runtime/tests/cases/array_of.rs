//! Array.of construction, definitions, and final strict length assignment.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn items_are_elements_including_single_numbers_and_preserve_identity() {
    check(
        "let a=Array.of(3);a.length===1 && a[0]===3 && Array.isArray(a) && Object.getPrototypeOf(a)===Array.prototype",
    );
    check(
        "let a=Array.of(undefined,null,true,1n);a.length===4 && Object.hasOwn(a,'0') && a[0]===undefined && a[1]===null && a[2]===true && a[3]===1n",
    );
    check("let a=Array.of();a!==Array.of() && a.length===0 && !Object.hasOwn(a,'0')");
    check("let o={valueOf:()=>{throw 7;}},a=Array.of(o,o);a[0]===o && a[1]===o");
    check(
        "let log='';function f(n){log+=n;return n;}let a=Array.of(f(1),f(2));log==='12' && a.join()==='1,2'",
    );
}

#[test]
fn nonconstructors_fall_back_without_being_called_or_coerced() {
    for receiver in [
        "undefined",
        "null",
        "false",
        "7",
        "1n",
        "'x'",
        "{}",
        "()=>{throw 7;}",
        "Array.of",
        "Array.of.bind(null)",
    ] {
        check(&format!(
            "let a=Array.of.call({receiver},1,2);Array.isArray(a) && a.join()==='1,2' && Object.getPrototypeOf(a)===Array.prototype"
        ));
    }
    check(
        "let o={valueOf:()=>{throw 7;}};Object.defineProperty(o,'prototype',{get:()=>{throw 8;}});Array.of.call(o,9)[0]===9",
    );
    check(
        "let f=Array.of,p=Array.prototype;Array=function(){throw 7;};Object.getPrototypeOf(f(9))===p",
    );
}

#[test]
fn constructors_receive_one_length_argument_and_their_result_is_used() {
    check(
        "let log='',target;function C(n){log+=n;target=new.target;this.n=n;this.argc=arguments.length;}let a=Array.of.call(C,7,8);a instanceof C && !Array.isArray(a) && a.n===2 && a.argc===1 && a.length===2 && a[0]===7 && a[1]===8 && log==='2' && target===C",
    );
    check(
        "let n,argc;function C(x){n=x;argc=arguments.length;}let a=Array.of.call(C);n===0 && argc===1 && a.length===0",
    );
    check(
        "let o={},target;function C(x,n){target=new.target;this.x=x;this.n=n;}let B=C.bind(o,'bound'),a=Array.of.call(B,7);a instanceof C && a.x==='bound' && a.n===1 && target===C && !Object.hasOwn(o,'x')",
    );
    check(
        "let o={};function C(){return o;}let a=Array.of.call(C,7);a===o && a[0]===7 && a.length===1",
    );
    check("function C(){return 7;}let a=Array.of.call(C,9);a instanceof C && a[0]===9");
    check(
        "let a=Array.of.call(Object,7,8);a instanceof Number && a.valueOf()===2 && a.length===2 && a[0]===7 && a[1]===8",
    );
    assert_eq!(
        Realm::default().eval("function C(){throw 7;}Array.of.call(C,1)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
}

#[test]
fn own_data_definitions_bypass_setters_and_replace_configurable_properties() {
    check(
        "Object.defineProperty(Array.prototype,'0',{set:()=>{throw 7;}});let a=Array.of(9),d=Object.getOwnPropertyDescriptor(a,'0');a[0]===9 && d.writable && d.enumerable && d.configurable",
    );
    check(
        "function C(){Object.defineProperty(this,'0',{value:1,writable:false,enumerable:false,configurable:true});Object.defineProperty(this,'1',{get:()=>{throw 7;},set:()=>{throw 8;},configurable:true});}Object.defineProperty(C.prototype,'2',{set:()=>{throw 9;}});let a=Array.of.call(C,7,8,9),d=Object.getOwnPropertyDescriptor(a,'0'),e=Object.getOwnPropertyDescriptor(a,'1');a[0]===7 && a[1]===8 && a[2]===9 && d.writable && d.enumerable && d.configurable && e.value===8 && e.writable && e.enumerable && e.configurable",
    );
}

#[test]
fn final_length_set_is_strict_even_when_empty_and_observes_inherited_setters() {
    for (args, expected) in [("", 0), (",7,8", 2)] {
        check(&format!(
            "let receiver,value,hits=0,seen;function C(){{}}Object.defineProperty(C.prototype,'length',{{set:function(n){{hits++;receiver=this;value=n;seen=this[1];}}}});let a=Array.of.call(C{args});hits===1 && receiver===a && value==={expected} && !Object.hasOwn(a,'length') && seen==={seen}",
            seen = if expected == 0 { "undefined" } else { "8" }
        ));
        assert!(matches!(Realm::default().eval(&format!("function C(){{Object.defineProperty(this,'length',{{value:0,writable:false}});}}Array.of.call(C{args})")),Err(Error::Exception{kind:ExceptionKind::TypeError,..})));
        assert_eq!(Realm::default().eval(&format!("function C(){{Object.defineProperty(this,'length',{{set:()=>{{throw 7;}}}});}}Array.of.call(C{args})")),Err(Error::Thrown(Value::Number(7.0))));
    }
}

#[test]
fn rejected_definitions_and_length_writes_preserve_prior_effects() {
    check(
        "let o;function C(){o=this;Object.defineProperty(this,'1',{value:0,writable:true,enumerable:true,configurable:false});}let caught=false;try{Array.of.call(C,7,8,9);}catch(e){caught=e instanceof TypeError;}caught && o[0]===7 && o[1]===0 && !Object.hasOwn(o,'2') && !Object.hasOwn(o,'length')",
    );
    check(
        "let o;function C(){o=this;Object.defineProperty(this,'length',{value:0,writable:false});}let caught=false;try{Array.of.call(C,7,8);}catch(e){caught=e instanceof TypeError;}caught && o[0]===7 && o[1]===8 && o.length===0",
    );
    check(
        "function C(){Object.preventExtensions(this);}let caught=false;try{Array.of.call(C,7);}catch(e){caught=e instanceof TypeError;}caught",
    );
    check(
        "let o=[];Object.defineProperty(o,'length',{writable:false});function C(){return o;}let caught=false;try{Array.of.call(C,7);}catch(e){caught=e instanceof TypeError;}caught && o.length===0 && !Object.hasOwn(o,'0')",
    );
    check(
        "let o=[1,2,3];function C(){return o;}Array.of.call(C,7)===o && o.length===1 && o[0]===7 && !Object.hasOwn(o,'1')",
    );
}

#[test]
fn metadata_nonconstructibility_and_intrinsic_gc_roots_are_standard() {
    check(
        "let f=Array.of,d=Object.getOwnPropertyDescriptor(Array,'of'),n=Object.getOwnPropertyDescriptor(f,'name'),l=Object.getOwnPropertyDescriptor(f,'length');d.value===f && d.writable && !d.enumerable && d.configurable && n.value==='of' && !n.writable && !n.enumerable && n.configurable && l.value===0 && !l.writable && !l.enumerable && l.configurable && f.prototype===undefined",
    );
    assert!(matches!(
        Realm::default().eval("new Array.of"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm
        .eval(
            "let f=Array.of,p=Array.prototype,o={},a=f(o);delete Array.of;delete globalThis.Array",
        )
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("a[0]===o && f(o)[0]===o && Object.getPrototypeOf(f())===p"),
        Ok(Value::Boolean(true))
    );
}
