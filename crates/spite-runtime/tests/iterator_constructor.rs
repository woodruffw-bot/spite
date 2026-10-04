//! Abstract Iterator construction and protected constructor accessors (27.1.3).

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn type_error(source: &str) {
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

#[test]
fn constructor_graph_metadata_and_existing_iterator_inheritance_are_standard() {
    check(
        "typeof Iterator==='function' && Iterator.name==='Iterator' && Iterator.length===0 && Object.getPrototypeOf(Iterator)===Function.prototype && Object.getPrototypeOf(Iterator.prototype)===Object.prototype && Iterator.prototype.constructor===Iterator && Iterator.toString()==='function Iterator() { [native code] }'",
    );
    check(
        "let p=Object.getOwnPropertyDescriptor(Iterator,'prototype'),g=Object.getOwnPropertyDescriptor(globalThis,'Iterator');p.value===Iterator.prototype && !p.writable && !p.enumerable && !p.configurable && g.value===Iterator && g.writable && !g.enumerable && g.configurable",
    );
    for (key, value) in [("name", "'Iterator'"), ("length", "0")] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Iterator,'{key}');d.value==={value} && !d.writable && !d.enumerable && d.configurable"
        ));
    }
    check(
        "let d=Object.getOwnPropertyDescriptor(Iterator.prototype,'constructor');!Object.hasOwn(d,'value') && !Object.hasOwn(d,'writable') && !d.enumerable && d.configurable && d.get.name==='get constructor' && d.get.length===0 && d.set.name==='set constructor' && d.set.length===1 && !Object.hasOwn(d.get,'prototype') && !Object.hasOwn(d.set,'prototype')",
    );
    check(
        "[].values() instanceof Iterator && 'x'[Symbol.iterator]() instanceof Iterator && (function(){return arguments[Symbol.iterator]() instanceof Iterator;})() && Iterator.prototype[Symbol.iterator]()===Iterator.prototype",
    );
    let mut realm = Realm::default();
    let Value::Object(constructor) = realm.eval("Iterator").unwrap() else {
        panic!("constructor")
    };
    let constructor = realm.inspect_object(&constructor).unwrap();
    assert!(constructor.is_callable() && constructor.is_constructor());
}

#[test]
fn abstract_calls_and_construction_reject_after_argument_evaluation() {
    for expression in [
        "Iterator(flag=3)",
        "new Iterator(flag=3)",
        "Iterator.call({},flag=3)",
        "new (Iterator.bind(null))(flag=3)",
        "Reflect.construct(Iterator,[flag=3])",
    ] {
        check(&format!(
            "let flag=0,caught=false;try{{{expression};}}catch(e){{caught=e instanceof TypeError;}}caught && flag===3"
        ));
    }
    check(
        "let I=Iterator;globalThis.Iterator=function(){};let caught=false;try{new I();}catch(e){caught=e instanceof TypeError;}caught",
    );
    for source in [
        "new (Object.getOwnPropertyDescriptor(Iterator.prototype,'constructor').get)",
        "new (Object.getOwnPropertyDescriptor(Iterator.prototype,'constructor').set)",
        "Reflect.construct(Iterator,[],()=>0)",
    ] {
        type_error(source);
    }
}

#[test]
fn distinct_newtargets_select_prototypes_without_calling_or_coercing_arguments() {
    check(
        "let calls=0;function F(){calls++;throw 7;}let p={},v={valueOf:()=>{throw 8;},toString:()=>{throw 9;}};F.prototype=p;let o=Reflect.construct(Iterator,[v],F);Object.getPrototypeOf(o)===p && calls===0 && Reflect.ownKeys(o).length===0",
    );
    for prototype in ["undefined", "null", "true", "1", "1n", "'x'", "Symbol()"] {
        check(&format!(
            "function F(){{throw 7;}}F.prototype={prototype};let o=Reflect.construct(Iterator,[],F);Object.getPrototypeOf(o)===Iterator.prototype && o instanceof Iterator && o[Symbol.iterator]()===o && o.next===undefined"
        ));
    }
    check(
        "let I=Iterator,p=I.prototype;globalThis.Iterator=null;delete p.constructor;function F(){}F.prototype=null;let o=Reflect.construct(I,[],F);Object.getPrototypeOf(o)===p && o instanceof I",
    );
}

#[test]
fn bound_forwarding_and_prototype_getters_preserve_newtarget_order() {
    check(
        "let B=Iterator.bind(null),caught=false;Object.defineProperty(B,'prototype',{get:()=>{throw 7;}});try{new B();}catch(e){caught=e instanceof TypeError;}caught",
    );
    check(
        "function F(){}let B=Iterator.bind(null);Object.getPrototypeOf(Reflect.construct(B,[],F))===F.prototype",
    );
    check(
        "let B=Iterator.bind(null);Object.getPrototypeOf(Reflect.construct(Iterator,[],B))===Iterator.prototype",
    );
    check(
        "function F(){throw 7;}let B=F.bind(null),p={},reads=0;Object.defineProperty(B,'prototype',{get:()=>{reads++;return p;}});let o=Reflect.construct(Iterator,[],B);reads===1 && Object.getPrototypeOf(o)===p",
    );
    assert_eq!(Realm::default().eval("function F(){}let B=F.bind(null);Object.defineProperty(B,'prototype',{get:()=>{throw 7;}});Reflect.construct(Iterator,[],B)"), Err(Error::Thrown(Value::Number(7.0))));
}

#[test]
fn constructor_getter_is_generic_and_setter_protects_the_intrinsic_home() {
    for receiver in [
        "undefined",
        "null",
        "true",
        "1",
        "1n",
        "'x'",
        "Symbol()",
        "Iterator.prototype",
    ] {
        let getter_receiver = if receiver == "Iterator.prototype" {
            "I.prototype"
        } else {
            receiver
        };
        check(&format!(
            "let I=Iterator,d=Object.getOwnPropertyDescriptor(I.prototype,'constructor');globalThis.Iterator=null;d.get.call({getter_receiver})===I"
        ));
        type_error(&format!(
            "let d=Object.getOwnPropertyDescriptor(Iterator.prototype,'constructor');d.set.call({receiver},'x')"
        ));
    }
    check(
        "let p=Iterator.prototype,get=Object.getOwnPropertyDescriptor(p,'constructor').get;let caught=false;try{p.constructor='x';}catch(e){caught=e instanceof TypeError;}caught && p.constructor===Iterator && get.call(null)===Iterator",
    );
    type_error(
        "let p=Iterator.prototype,set=Object.getOwnPropertyDescriptor(p,'constructor').set;delete p.constructor;set.call(p,7)",
    );
}

#[test]
fn constructor_setter_bypasses_inherited_properties_and_strictly_updates_own_ones() {
    check(
        "let p=Iterator.prototype,child=Object.create(p),v={toString:()=>{throw 7;}};child.constructor=v;let d=Object.getOwnPropertyDescriptor(child,'constructor');d.value===v && d.writable && d.enumerable && d.configurable && p.constructor===Iterator",
    );
    check(
        "let set=Object.getOwnPropertyDescriptor(Iterator.prototype,'constructor').set,parent={};Object.defineProperty(parent,'constructor',{get:()=>{throw 7;},set:()=>{throw 8;}});let o=Object.create(parent);set.call(o,9)===undefined && Object.hasOwn(o,'constructor') && o.constructor===9",
    );
    check(
        "let set=Object.getOwnPropertyDescriptor(Iterator.prototype,'constructor').set,parent={};Object.defineProperty(parent,'constructor',{value:7});let o=Object.create(parent);set.call(o)===undefined && Object.hasOwn(o,'constructor') && o.constructor===undefined && parent.constructor===7",
    );
    check(
        "let set=Object.getOwnPropertyDescriptor(Iterator.prototype,'constructor').set,o={};Object.defineProperty(o,'constructor',{value:7,writable:true});Object.preventExtensions(o);set.call(o,8);let d=Object.getOwnPropertyDescriptor(o,'constructor');d.value===8 && d.writable && !d.enumerable && !d.configurable",
    );
    check(
        "let set=Object.getOwnPropertyDescriptor(Iterator.prototype,'constructor').set,o={},receiver,value,count=0,v={};Object.defineProperty(o,'constructor',{get:()=>{throw 7;},set:function(x){'use strict';receiver=this;value=x;count++;}});set.call(o,v)===undefined && receiver===o && value===v && count===1",
    );
    for descriptor in ["{value:7}", "{get:()=>7}"] {
        type_error(&format!(
            "let set=Object.getOwnPropertyDescriptor(Iterator.prototype,'constructor').set,o={{}};Object.defineProperty(o,'constructor',{descriptor});set.call(o,8)"
        ));
    }
    type_error(
        "let set=Object.getOwnPropertyDescriptor(Iterator.prototype,'constructor').set;set.call(Object.preventExtensions({}),7)",
    );
    assert_eq!(Realm::default().eval("let set=Object.getOwnPropertyDescriptor(Iterator.prototype,'constructor').set,o={};Object.defineProperty(o,'constructor',{set:()=>{throw 7;}});set.call(o,8)"), Err(Error::Thrown(Value::Number(7.0))));
}

#[test]
fn incomplete_prototype_remains_a_host_gap_and_skips_handlers() {
    for expression in [
        "Reflect.ownKeys(Iterator.prototype)",
        "function F(){}let B=F.bind(null);Object.defineProperty(B,'prototype',{get:()=>Reflect.ownKeys(Iterator.prototype)});Reflect.construct(Iterator,[],B)",
    ] {
        let mut realm = Realm::default();
        realm.eval("let flag=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{{expression};}}catch{{flag=1;}}finally{{flag=2;}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{expression}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    check(
        "'from' in Iterator && 'concat' in Iterator && !('zip' in Iterator) && Iterator.zip===undefined && Iterator.zipKeyed===undefined",
    );
}

#[test]
fn constructor_and_accessors_survive_deleted_public_links_and_collection() {
    let mut realm = Realm::default();
    realm.eval("let I=Iterator,p=I.prototype,d=Object.getOwnPropertyDescriptor(p,'constructor'),get=d.get,set=d.set;d=null;delete p.constructor;delete globalThis.Iterator;").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(realm.eval("get.call(null)===I && Object.getPrototypeOf(I)===Function.prototype && set.call({},7)===undefined && typeof Iterator==='undefined'"), Ok(Value::Boolean(true)));
}
