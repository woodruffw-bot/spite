//! Heritage, SuperCall, and derived this/return semantics (15.7, 13.3.7).

use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn superclass_and_prototype_are_checked_in_order() {
    check(
        "let log='',B=function(){}.bind(null);Object.defineProperty(B,'prototype',{get(){log+='P';return null;}});class C extends (log+='H',B){[(log+='K','m')](){}}log==='HPK' && Object.getPrototypeOf(C)===B && Object.getPrototypeOf(C.prototype)===null",
    );
    for base in ["undefined", "7", "true", "{}", "()=>{}", "Math.abs"] {
        check(&format!(
            "let caught=false;try{{class C extends ({base}){{}}}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    check(
        "function B(){}B.prototype=7;let reached=false,caught=false;try{class C extends B{[(reached=true,'x')](){}}}catch(e){caught=e instanceof TypeError;}caught && !reached",
    );
    check(
        "let caught=false;try{class C extends C{}}catch(e){caught=e instanceof ReferenceError;}caught",
    );
    check(
        "let x=function(){};let caught=false;try{let C=class x extends x{};}catch(e){caught=e instanceof ReferenceError;}caught && typeof x==='function'",
    );
}

#[test]
fn default_constructors_forward_lists_without_array_iteration() {
    check(
        "class B{constructor(a,b){this.sum=a+b;this.target=new.target;}}class D extends B{}let d=new D(3,4);d.sum===7 && d.target===D && d instanceof D && d instanceof B && D.length===0",
    );
    check(
        "class B{constructor(a,b){this.sum=a+b;}}class D extends B{}Array.prototype[Symbol.iterator]=function(){throw 7;};new D(3,4).sum===7",
    );
    check("let C=class{};for(let i=0;i<100;i++){C=class extends C{};}new C instanceof C");
    check("class B{constructor(){return {x:7};}}class D extends B{}new D().x===7");
}

#[test]
fn explicit_super_initializes_this_and_preserves_new_target() {
    check(
        "class B{constructor(x){this.x=x;this.target=new.target;}}class D extends B{constructor(x){super(x+1);this.y=2;}}let d=new D(6);d.x===7 && d.y===2 && d.target===D && Object.getPrototypeOf(d)===D.prototype",
    );
    check(
        "class B{constructor(x){this.x=x;}}class D extends B{constructor(){return super(7);}}new D().x===7",
    );
    check(
        "class B{}class D extends B{constructor(){super();}}function N(){}let x=Reflect.construct(D,[],N);Object.getPrototypeOf(x)===N.prototype && x instanceof N",
    );
}

#[test]
fn derived_return_rules_distinguish_objects_undefined_and_other_primitives() {
    check("let o={};class D extends Object{constructor(){return o;}}new D===o");
    check(
        "class D extends Object{constructor(){}}let caught=false;try{new D;}catch(e){caught=e instanceof ReferenceError;}caught",
    );
    for value in ["null", "3", "true", "'x'", "3n", "Symbol()"] {
        for init in ["", "super();"] {
            check(&format!(
                "class D extends Object{{constructor(){{{init}return {value};}}}}let caught=false;try{{new D;}}catch(e){{caught=e instanceof TypeError;}}caught"
            ));
        }
    }
    check("class D extends Object{constructor(){super();return undefined;}}new D instanceof D");
}

#[test]
fn this_and_super_properties_throw_before_initialization_and_computed_keys() {
    check(
        "let log='',caught=false;class D extends Object{constructor(){try{super[(log+='K','x')];}catch(e){caught=e instanceof ReferenceError;}super();}}new D;caught && log===''",
    );
    check(
        "let caught=false;class D extends Object{constructor(x=this){super();}}try{new D;}catch(e){caught=e instanceof ReferenceError;}caught",
    );
    check(
        "let caught=false;class D extends Object{constructor(){try{(()=>this)();}catch(e){caught=e instanceof ReferenceError;}super();}}new D;caught",
    );
}

#[test]
fn repeated_super_still_evaluates_arguments_and_constructs_before_bind_rejection() {
    check(
        "let log='';class B{constructor(x){log+=x;}}class D extends B{constructor(){super('A');let first=this;try{super((log+='E','B'));}catch(e){if(!(e instanceof ReferenceError))throw e;}this.same=this===first;}}let d=new D;log==='AEB' && d.same",
    );
    check(
        "let log='';class D extends Object{constructor(){try{super(super(),log+='later');}catch(e){if(!(e instanceof ReferenceError))throw e;}}}new D;log==='later'",
    );
}

#[test]
fn super_constructor_is_captured_before_arguments_and_validated_after_them() {
    check(
        "let log='';class A{constructor(){log+='A';}}class B{constructor(){log+='B';}}class D extends A{constructor(){super(Object.setPrototypeOf(D,B));}}new D;log==='A' && Object.getPrototypeOf(D)===B",
    );
    check(
        "let log='',caught=false;class D extends Object{constructor(){super(log+='E');}}Object.setPrototypeOf(D,{});try{new D;}catch(e){caught=e instanceof TypeError;}caught && log==='E'",
    );
    check(
        "let caught=false;class D extends Object{constructor(){super((function(){throw 7;})());}}Object.setPrototypeOf(D,{});try{new D;}catch(e){caught=e===7;}caught",
    );
}

#[test]
fn arrows_and_direct_eval_inherit_derived_constructor_state() {
    check(
        "class B{constructor(x){this.x=x;}}class D extends B{constructor(f=()=>super(7)){f();}}new D().x===7",
    );
    check(
        "class D extends Object{constructor(){eval('super()');this.target=eval('new.target');}}new D().target===D",
    );
    check("class D extends Object{constructor(){eval('()=>super()')();}}new D instanceof D");
    check(
        "let saved;class D extends Object{constructor(){saved=()=>super();return {};}}new D;let d=saved();d instanceof D && (function(){try{saved();}catch(e){return e instanceof ReferenceError;}})()",
    );
    check(
        "class D extends Object{constructor(){let caught=false;try{eval('function f(){super();}');}catch(e){caught=e instanceof SyntaxError;}if(!caught)throw 7;super();}}new D instanceof D",
    );
}

#[test]
fn extends_null_requires_explicit_object_return_without_super() {
    check(
        "class D extends null{constructor(){return Object.create(new.target.prototype);}}let d=new D;Object.getPrototypeOf(D)===Function.prototype && Object.getPrototypeOf(D.prototype)===null && d instanceof D",
    );
    check(
        "class D extends null{}let caught=false;try{new D;}catch(e){caught=e instanceof TypeError;}caught",
    );
    check(
        "class D extends null{constructor(){super();}}let caught=false;try{new D;}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn inherited_methods_and_static_super_keep_the_actual_receiver() {
    check(
        "class B{m(){return this.x;}static m(){return this.x;}}class D extends B{m(){return super.m()+1;}static m(){return super.m()+1;}}let d=new D;d.x=6;D.x=8;d.m()===7 && D.m()===9 && D.prototype.m.call({x:2})===3",
    );
    check(
        "class B{get x(){return this.y;}set x(v){this.y=v;}}class D extends B{constructor(){super();super.x=7;}get x(){return super.x+1;}}new D().x===8",
    );
}

#[test]
fn native_superclasses_initialize_internal_slots_with_the_derived_prototype() {
    check(
        "class D extends Array{}let d=new D(1,2);Array.isArray(d) && d instanceof D && d.length===2 && d[1]===2",
    );
    check("class D extends Map{}let d=new D([[1,2]]);d instanceof D && d.get(1)===2 && d.size===1");
    check("class D extends Set{}let d=new D([1,2]);d instanceof D && d.size===2 && d.has(2)");
    check("class D extends Error{}let d=new D('x');d instanceof D && d.message==='x'");
}

#[test]
fn escaped_super_arrow_environments_retain_constructors_through_collection() {
    let mut realm = Realm::default();
    realm.eval("let f=(function(){let arrow;class B{constructor(){this.x=7;}}class D extends B{constructor(){arrow=()=>super();return {};}}new D;return arrow;})();").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f().x"), Ok(Value::Number(7.0)));
    realm.eval("f=undefined").unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn superclass_failures_restore_context_and_leave_this_uninitialized() {
    check(
        "class B{constructor(){throw 7;}}class D extends B{constructor(){let caught=false;try{super();}catch(e){caught=e===7;}try{this;}catch(e){if(caught && e instanceof ReferenceError)return {x:9};}throw 8;}}new D().x===9",
    );
    check(
        "let log='';class D extends Object{constructor(){super();try{super(super(),log+='later');}catch(e){if(!(e instanceof ReferenceError))throw e;}}}new D;log===''",
    );
    let mut realm = Realm::default();
    realm
        .eval("class D extends Object{constructor(){return null;}}")
        .unwrap();
    assert_eq!(
        realm.eval("try{new D;}catch(e){e instanceof TypeError;}"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("after=3"), Ok(Value::Number(3.0)));
    realm
        .eval(
            "let B=function(){}.bind(null);Object.defineProperty(B,'prototype',{get(){throw 7;}})",
        )
        .unwrap();
    assert_eq!(
        realm.eval("try{class Failed extends B{}}catch(e){e===7;}"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("next=4"), Ok(Value::Number(4.0)));
}
