//! WeakSet identity, constructor ordering, weak reachability, and branded methods.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

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
fn objects_and_non_registered_symbols_preserve_identity_without_coercion() {
    check(
        "let a={toString(){throw 7;},valueOf(){throw 8;}},b={},x=Symbol('x'),y=Symbol('x'),s=new WeakSet([a,b,x,y,x]);s.has(a) && s.has(b) && s.has(x) && s.has(y) && !s.has({}) && !s.has(Symbol('x')) && s.add(a)===s && s.add(x)===s && s.delete(a) && !s.delete(a) && s.delete(x) && !s.delete(x) && s.has(y) && s.add(a)===s && s.has(a)",
    );
    check(
        "let s=new WeakSet([Symbol.iterator]);s.has(Symbol.iterator) && s.delete(Symbol.iterator)",
    );
    check(
        "let s=new WeakSet([Object(Symbol()),Object(1)]),k=Object.freeze({});Object.freeze(s);s.add(k)===s && s.has(k) && s.delete(k) && Object.isFrozen(s)",
    );
}

#[test]
fn registered_symbols_and_other_primitives_are_rejected_without_conversion() {
    for key in [
        "undefined",
        "null",
        "false",
        "7",
        "1n",
        "'x'",
        "Symbol.for('weak-set-registered')",
    ] {
        type_error(&format!("new WeakSet().add({key})"));
        check(&format!(
            "let s=new WeakSet();!s.has({key}) && !s.delete({key})"
        ));
    }
    check(
        "let x=Symbol('weak-set-registered'),r=Symbol.for('weak-set-registered'),s=new WeakSet([x]);s.has(x) && !s.has(r)",
    );
    check(
        "Symbol.keyFor=function(){throw 7;};let x=Symbol(),s=new WeakSet([x]);s.has(x) && !s.has(Symbol.for('weak-set-registered'))",
    );
}

#[test]
fn receivers_require_their_own_slot_before_key_validation() {
    for receiver in [
        "undefined",
        "null",
        "1",
        "Symbol()",
        "{}",
        "WeakSet.prototype",
        "new Set()",
        "new Map()",
        "Object.create(new WeakSet())",
    ] {
        for method in ["add", "has", "delete"] {
            type_error(&format!("WeakSet.prototype.{method}.call({receiver},null)"));
        }
    }
}

#[test]
fn construction_orders_prototype_adder_and_iterator_and_caches_the_adder() {
    type_error("WeakSet()");
    check(
        "let effects=0,caught=false;try{WeakSet(effects++);}catch(e){caught=e instanceof TypeError;}caught && effects===1",
    );
    check(
        "let trace='',key={},p=Object.create(WeakSet.prototype);Object.defineProperty(p,'add',{get(){trace+='a';return function(v){trace+='c';if(arguments.length!==1)throw 7;WeakSet.prototype.add.call(this,v);};}});function C(){}C.prototype=p;let items={[Symbol.iterator](){trace+='i';let n=0;return {get next(){trace+='n';return function(){return n++===0?{value:key}:{done:true};};}};}};let s=Reflect.construct(WeakSet,[items],C);trace==='ainc' && Object.getPrototypeOf(s)===p && s.has(key)",
    );
    check(
        "let p=Object.create(WeakSet.prototype);Object.defineProperty(p,'add',{get(){throw 7;}});function C(){}C.prototype=p;Object.getPrototypeOf(Reflect.construct(WeakSet,[null],C))===p && Object.getPrototypeOf(Reflect.construct(WeakSet,[],C))===p",
    );
    check(
        "let touched=false,p={add:1};function C(){}C.prototype=p;let items={get [Symbol.iterator](){touched=true;throw 7;}},caught=false;try{Reflect.construct(WeakSet,[items],C);}catch(e){caught=e instanceof TypeError;}caught && !touched",
    );
    check(
        "let calls=0,a={},b={},add=WeakSet.prototype.add;WeakSet.prototype.add=function(v){calls++;WeakSet.prototype.add=null;return add.call(this,v);};let s=new WeakSet([a,b]);calls===2 && s.has(a) && s.has(b)",
    );
    check(
        "function C(){}C.prototype=1;Object.getPrototypeOf(Reflect.construct(WeakSet,[],C))===WeakSet.prototype",
    );
}

#[test]
fn construction_closes_adder_failures_and_preserves_escaped_partial_sets() {
    check(
        "let key={},escaped,closed=0,n=0,add=WeakSet.prototype.add;WeakSet.prototype.add=function(v){escaped=this;return add.call(this,v);};let i={next(){return {value:n++===0?key:1};},return(){closed++;throw 7;}},caught=false;try{new WeakSet({[Symbol.iterator](){return i;}});}catch(e){caught=e instanceof TypeError;}caught && closed===1 && escaped.has(key)",
    );
    check(
        "let marker={},closed=0;WeakSet.prototype.add=function(){throw marker;};let i={next(){return {value:{}};},return(){closed++;throw 7;}},caught=false;try{new WeakSet({[Symbol.iterator](){return i;}});}catch(e){caught=e===marker;}caught && closed===1",
    );
    for step in [
        "throw marker;",
        "return {get done(){throw marker;}};",
        "return {get value(){throw marker;}};",
    ] {
        check(&format!(
            "let marker={{}},closed=0,i={{next(){{{step}}},return(){{closed++;return {{}};}}}},caught=false;try{{new WeakSet({{[Symbol.iterator](){{return i;}}}});}}catch(e){{caught=e===marker;}}caught && closed===0"
        ));
    }
}

#[test]
fn subclasses_bound_construction_and_custom_new_target_keep_the_internal_slot() {
    check(
        "let key={},symbol=Symbol();class C extends WeakSet{#n=this.has(key);read(){return this.#n;}}let s=new C([key,symbol]);s instanceof C && s instanceof WeakSet && s.read() && s.has(symbol)",
    );
    check(
        "let key={};class C extends WeakSet{constructor(items){super(items);this.n=this.has(key);}}let s=new C([key]);s.n && s.delete(key)",
    );
    check("let key={},B=WeakSet.bind(null,[key]),s=new B();s instanceof WeakSet && s.has(key)");
    check(
        "let key={},p={};function C(){}C.prototype=p;let s=Reflect.construct(WeakSet,[],C);Object.getPrototypeOf(s)===p && WeakSet.prototype.add.call(s,key)===s && WeakSet.prototype.has.call(s,key)",
    );
}

#[test]
fn intrinsic_metadata_reflection_and_integrity_are_complete() {
    check(
        "let c=Object.getOwnPropertyDescriptor(globalThis,'WeakSet'),p=Object.getOwnPropertyDescriptor(WeakSet,'prototype'),t=Object.getOwnPropertyDescriptor(WeakSet.prototype,Symbol.toStringTag);WeakSet.length===0 && WeakSet.name==='WeakSet' && Object.getPrototypeOf(WeakSet)===Function.prototype && Object.getPrototypeOf(WeakSet.prototype)===Object.prototype && WeakSet.prototype.constructor===WeakSet && c.value===WeakSet && c.writable && !c.enumerable && c.configurable && p.value===WeakSet.prototype && !p.writable && !p.enumerable && !p.configurable && t.value==='WeakSet' && !t.writable && !t.enumerable && t.configurable",
    );
    check(
        "Object.getOwnPropertyNames(WeakSet).join()==='length,name,prototype' && Object.getOwnPropertySymbols(WeakSet).length===0 && Object.getOwnPropertyNames(WeakSet.prototype).join()==='constructor,add,delete,has' && Object.getOwnPropertySymbols(WeakSet.prototype)[0]===Symbol.toStringTag && Reflect.ownKeys(new WeakSet()).length===0 && Object.prototype.toString.call(new WeakSet())==='[object WeakSet]' && Object.keys(WeakSet.prototype).length===0",
    );
    for method in ["add", "delete", "has"] {
        check(&format!(
            "let f=WeakSet.prototype.{method},d=Object.getOwnPropertyDescriptor(WeakSet.prototype,'{method}');f.name==='{method}' && f.length===1 && !Object.hasOwn(f,'prototype') && d.writable && !d.enumerable && d.configurable"
        ));
        type_error(&format!("new WeakSet.prototype.{method}()"));
    }
    check(
        "Object.freeze(WeakSet);Object.freeze(WeakSet.prototype);let key={},s=new WeakSet([key]);Object.isFrozen(WeakSet) && Object.isFrozen(WeakSet.prototype) && s.has(key) && s.delete(key)",
    );
    check(
        "delete WeakSet.prototype[Symbol.toStringTag];Object.prototype.toString.call(new WeakSet())==='[object Object]'",
    );
}

#[test]
fn collection_keeps_live_keys_usable_but_does_not_retain_unreachable_key_cycles() {
    let mut realm = Realm::default();
    realm
        .eval("let set=new WeakSet(),key={},symbol=Symbol();set.add(key).add(symbol);key.set=set;")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("set.has(key) && set.has(symbol)"),
        Ok(Value::Boolean(true))
    );
    realm.eval("key=undefined;symbol=undefined;").unwrap();
    assert!(realm.collect(usize::MAX).unwrap().reclaimed >= 1);
    assert_eq!(
        realm.eval("let next={};set.add(next);set.has(next) && !set.has({})"),
        Ok(Value::Boolean(true))
    );
    realm.eval("next=undefined;set=undefined;").unwrap();
    assert!(realm.collect(usize::MAX).unwrap().reclaimed >= 2);
}

#[test]
fn unlimited_defaults_support_large_collections_and_opted_in_abort_skips_cleanup() {
    check(
        "let keys=Array.from({length:4000},()=>({})),s=new WeakSet(keys);keys.every(k=>s.has(k)) && keys.every(k=>s.delete(k)) && keys.every(k=>!s.has(k))",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(10_000),
        ..Limits::default()
    });
    realm
        .eval("var cleaned=0;var i={next(){return {value:{}};},return(){cleaned++;return {};}};")
        .unwrap();
    assert!(matches!(
        realm.eval("try{new WeakSet({[Symbol.iterator](){return i;}});}catch{cleaned++;}finally{cleaned++;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("cleaned"), Ok(Value::Number(0.0)));
}

#[test]
fn reentrant_constructors_and_adders_use_the_existing_native_stack_guards() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            for source in [
                "Object.defineProperty(WeakSet.prototype,'add',{get(){return new WeakSet([]);}});new WeakSet([]);",
                "WeakSet.prototype.add=function(){return new WeakSet([{}]);};new WeakSet([{}]);",
                "class C extends WeakSet{constructor(){super();new C;}}new C;",
            ] {
                assert!(
                    matches!(Realm::default().eval(source), Err(Error::Limit { .. })),
                    "{source}"
                );
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
