//! Object-key WeakMap semantics and explicit Symbol-key staging boundaries (24.3).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

const METHODS: [&str; 6] = [
    "delete",
    "get",
    "getOrInsert",
    "getOrInsertComputed",
    "has",
    "set",
];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn type_error(source: &str) {
    let result = Realm::default().eval(source);
    assert!(
        matches!(
            &result,
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ),
        "{source}: {result:?}"
    );
}

#[test]
fn object_keys_use_identity_without_coercion_and_missing_differs_from_present_undefined() {
    check(
        "let a={get [Symbol.toPrimitive](){throw 7;},valueOf(){throw 8;},toString(){throw 9;}},b={},m=new WeakMap([[a,1],[b,2],[a,undefined]]);m.has(a) && m.get(a)===undefined && m.get(b)===2 && !m.has({}) && m.get({})===undefined && m.set(a,3)===m && m.get(a)===3 && m.delete(a) && !m.delete(a) && !m.has(a) && m.has(b) && m.set(a,4)===m && m.get(a)===4",
    );
    check(
        "let a=Object(Symbol('x')),b=Object(Symbol('x')),c=Object(Symbol.for('registered')),f=function(){},m=new WeakMap([[a,1],[b,2],[c,3],[f,4]]);m.get(a)===1 && m.get(b)===2 && m.get(c)===3 && m.get(f)===4 && !m.has(Object(Symbol.for('registered')))",
    );
    check(
        "let key={},m=new WeakMap([[key]]);m.has(key) && m.get(key)===undefined && m.set(key)===m && m.has(key)",
    );
}

#[test]
fn values_preserve_every_primitive_and_object_identity_without_conversion() {
    for value in [
        "undefined",
        "null",
        "false",
        "true",
        "NaN",
        "-0",
        "1n",
        "'value'",
        "Symbol('value')",
        "Symbol.for('value')",
        "{}",
        "function(){}",
        "new Map()",
    ] {
        check(&format!(
            "let key={{}},value={value},m=new WeakMap();m.set(key,value)===m && m.has(key) && Object.is(m.get(key),value)"
        ));
    }
}

#[test]
fn invalid_primitives_and_registered_symbols_keep_their_language_defined_outcomes() {
    for key in [
        "undefined",
        "null",
        "false",
        "7",
        "-0",
        "NaN",
        "1n",
        "'x'",
        "Symbol.for('weak-map-registered')",
    ] {
        check(&format!(
            "let m=new WeakMap();m.get({key})===undefined && !m.has({key}) && !m.delete({key})"
        ));
        for expression in [
            format!("new WeakMap().set({key},7)"),
            format!("new WeakMap().getOrInsert({key},7)"),
            format!("new WeakMap().getOrInsertComputed({key},()=>7)"),
        ] {
            type_error(&expression);
        }
    }
    check(
        "Symbol.keyFor=function(){throw 7;};let registered=Symbol.for('weak-map-registered'),m=new WeakMap();m.get(registered)===undefined && !m.has(registered) && !m.delete(registered)",
    );
}

#[test]
fn receiver_brand_checks_precede_valid_symbol_unsupported_and_callback_handling() {
    for receiver in [
        "undefined",
        "null",
        "1",
        "1n",
        "'x'",
        "Symbol()",
        "{}",
        "WeakMap.prototype",
        "new Map()",
        "new WeakSet()",
        "Object.create(new WeakMap())",
    ] {
        for method in METHODS {
            type_error(&format!(
                "WeakMap.prototype.{method}.call({receiver},Symbol('key'),()=>7)"
            ));
        }
    }
}

#[test]
fn valid_weak_symbols_are_explicit_host_unsupported_without_running_language_handlers() {
    for key in ["Symbol('key')", "Symbol.iterator"] {
        for method in METHODS {
            let arguments = if method == "getOrInsertComputed" {
                "key,()=>{calls++;return 7;}"
            } else {
                "key,7"
            };
            let mut realm = Realm::default();
            realm.eval(&format!("let key={key},m=new WeakMap(),flag=0,calls=0;Symbol.keyFor=function(){{throw 8;}}")).unwrap();
            assert!(
                matches!(
                    realm.eval(&format!(
                        "try{{m.{method}({arguments});}}catch{{flag=1;}}finally{{flag=2;}}"
                    )),
                    Err(Error::Unsupported { .. })
                ),
                "{method} / {key}"
            );
            assert_eq!(
                realm.eval("flag===0 && calls===0"),
                Ok(Value::Boolean(true))
            );
        }
        type_error(&format!("new WeakMap().getOrInsertComputed({key},null)"));
        type_error(&format!(
            "new WeakMap().getOrInsertComputed({key},{{valueOf(){{throw 7;}}}})"
        ));
    }
}

#[test]
fn upserts_return_existing_undefined_and_validate_computed_callbacks_before_lookup() {
    check(
        "let key={},other={},m=new WeakMap([[key,undefined]]),calls=0,value={valueOf(){throw 7;}};m.getOrInsert(key,value)===undefined && m.getOrInsertComputed(key,()=>{calls++;return 8;})===undefined && calls===0 && m.getOrInsert(other)===undefined && m.has(other) && m.get(other)===undefined",
    );
    for callback in ["undefined", "null", "7", "{}", "Symbol()"] {
        type_error(&format!(
            "let key={{}},m=new WeakMap([[key,7]]);m.getOrInsertComputed(key,{callback})"
        ));
    }
    check(
        "let key={},m=new WeakMap([[key,7]]);class C{}m.getOrInsertComputed(key,C)===7 && m.get(key)===7",
    );
    type_error("class C{}new WeakMap().getOrInsertComputed({},C)");
}

#[test]
fn computed_callbacks_receive_exactly_the_original_key_with_undefined_this() {
    check(
        "let key={toString(){throw 7;}},value={valueOf(){throw 8;}},m=new WeakMap(),calls=0,valid=false;let result=m.getOrInsertComputed(key,function(k){'use strict';calls++;valid=this===undefined && arguments.length===1 && k===key;return value;});valid && calls===1 && result===value && m.has(key) && m.get(key)===value",
    );
    check(
        "let key={},value=Symbol('value'),m=new WeakMap();m.getOrInsertComputed(key,()=>value)===value && m.get(key)===value",
    );
}

#[test]
fn computed_upsert_rechecks_after_reentrant_insert_delete_and_dense_relocation() {
    check(
        "let a={},b={},key={},other={},m=new WeakMap([[a,1],[b,2]]),calls=0;let result=m.getOrInsertComputed(key,()=>{calls++;m.set(key,'temporary');m.delete(a);m.delete(b);m.set(other,7);return 8;});result===8 && calls===1 && m.get(key)===8 && m.get(other)===7 && !m.has(a) && !m.has(b)",
    );
    check(
        "let key={},other={},m=new WeakMap();let result=m.getOrInsertComputed(key,()=>{m.set(key,1);m.delete(key);m.set(other,7);return undefined;});result===undefined && m.has(key) && m.get(key)===undefined && m.get(other)===7",
    );
    check(
        "let key={},m=new WeakMap(),calls=0;let result=m.getOrInsertComputed(key,()=>{calls++;let nested=m.getOrInsertComputed(key,()=>{calls++;return 3;});return nested+2;});result===5 && calls===2 && m.get(key)===5",
    );
}

#[test]
fn upsert_storage_does_not_call_public_get_has_or_set_overrides() {
    check(
        "let key={},other={},m=new WeakMap(),get=WeakMap.prototype.get;m.get=function(){throw 7;};m.has=function(){throw 8;};m.set=function(){throw 9;};let first=m.getOrInsert(key,3),second=m.getOrInsertComputed(other,()=>{Object.defineProperty(m,'set',{get(){throw 10;}});return 4;});first===3 && second===4 && get.call(m,key)===3 && get.call(m,other)===4",
    );
}

#[test]
fn callback_throws_keep_original_values_and_only_the_callbacks_own_prior_effects() {
    check(
        "let key={},other={},marker={},m=new WeakMap(),caught=false;try{m.getOrInsertComputed(key,()=>{m.set(other,7);throw marker;});}catch(e){caught=e===marker;}caught && !m.has(key) && m.get(other)===7",
    );
    check(
        "let key={},m=new WeakMap(),caught=false;try{m.getOrInsertComputed(key,()=>{m.set(key,3);throw 7;});}catch(e){caught=e===7;}caught && m.get(key)===3",
    );
}

#[test]
fn constructor_requires_new_and_orders_new_target_adder_iterator_and_entry_reads() {
    type_error("WeakMap()");
    check(
        "let effects=0,caught=false;try{WeakMap(effects++);}catch(e){caught=e instanceof TypeError;}caught && effects===1",
    );
    check(
        "let trace='',key={},p=Object.create(WeakMap.prototype);Object.defineProperty(p,'set',{get(){trace+='s';return function(k,v){'use strict';trace+='a';if(arguments.length!==2 || k!==key || v!==7)throw 8;WeakMap.prototype.set.call(this,k,v);};}});function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){trace+='p';return p;}});let items={[Symbol.iterator](){trace+='i';let used=false;return {get next(){trace+='n';return function(){if(used)return {done:true};used=true;return {value:{get 0(){trace+='k';return key;},get 1(){trace+='v';return 7;}}};};}};}};let m=Reflect.construct(WeakMap,[items],N);trace==='psinkva' && Object.getPrototypeOf(m)===p && m.get(key)===7",
    );
    check(
        "let p=Object.create(WeakMap.prototype);Object.defineProperty(p,'set',{get(){throw 7;}});function C(){}C.prototype=p;Object.getPrototypeOf(Reflect.construct(WeakMap,[],C))===p && Object.getPrototypeOf(Reflect.construct(WeakMap,[undefined],C))===p && Object.getPrototypeOf(Reflect.construct(WeakMap,[null],C))===p",
    );
    check(
        "let touched=false,p=Object.create(WeakMap.prototype);p.set=null;function C(){}C.prototype=p;let items={get [Symbol.iterator](){touched=true;throw 7;}},caught=false;try{Reflect.construct(WeakMap,[items],C);}catch(e){caught=e instanceof TypeError;}caught && !touched",
    );
}

#[test]
fn constructor_captures_the_adder_once_and_does_not_prevalidate_custom_adder_keys() {
    check(
        "let key={},other={},calls=0,set=WeakMap.prototype.set;WeakMap.prototype.set=function(k,v){calls++;if(calls===1)WeakMap.prototype.set=function(){throw 7;};return set.call(this,k,v);};let m=new WeakMap([[key,1],[other,2]]);calls===2 && m.get(key)===1 && m.get(other)===2",
    );
    check(
        "let registered=Symbol.for('weak-map-custom'),fresh=Symbol('fresh'),keys=[],values=[],valid=true;class C extends WeakMap{set(k,v){valid=valid && arguments.length===2 && WeakMap.prototype.has.call(this,{})===false;keys.push(k);values.push(v);return 7;}}let m=new C([[1,2],[registered,3],[fresh,4]]);valid && m instanceof C && keys.length===3 && keys[0]===1 && keys[1]===registered && keys[2]===fresh && values.join()==='2,3,4'",
    );
    type_error("new WeakMap({0:[{},7],length:1})");
    type_error("new WeakMap(7)");
}

#[test]
fn entry_get_and_adder_failures_close_once_while_iterator_step_failures_do_not() {
    for entry in [
        "7",
        "{get 0(){throw marker;}}",
        "{get 0(){log+='k';return {};},get 1(){throw marker;}}",
    ] {
        let expected = if entry == "7" {
            "e instanceof TypeError"
        } else {
            "e===marker"
        };
        check(&format!(
            "let marker={{}},log='',closed=0,i={{next(){{return {{value:{entry}}};}},return(){{closed++;throw 9;}}}},caught=false;try{{new WeakMap({{[Symbol.iterator](){{return i;}}}});}}catch(e){{caught={expected};}}caught && closed===1"
        ));
    }
    check(
        "let marker={},closed=0,calls=0;WeakMap.prototype.set=function(){calls++;throw marker;};let i={next(){return {value:[{},7]};},return(){closed++;return 1;}},caught=false;try{new WeakMap({[Symbol.iterator](){return i;}});}catch(e){caught=e===marker;}caught && calls===1 && closed===1",
    );
    for step in [
        "throw marker;",
        "return {get done(){throw marker;}};",
        "return {get value(){throw marker;}};",
    ] {
        check(&format!(
            "let marker={{}},closed=0,i={{next(){{{step}}},return(){{closed++;return {{}};}}}},caught=false;try{{new WeakMap({{[Symbol.iterator](){{return i;}}}});}}catch(e){{caught=e===marker;}}caught && closed===0"
        ));
    }
}

#[test]
fn invalid_key_constructor_failure_preserves_earlier_entries_and_language_close_effects() {
    check(
        "let key={},escaped,closed=0,n=0,set=WeakMap.prototype.set;WeakMap.prototype.set=function(k,v){escaped=this;return set.call(this,k,v);};let i={next(){return {value:n++===0?[key,7]:[Symbol.for('registered'),8]};},return(){closed++;throw 9;}},caught=false;try{new WeakMap({[Symbol.iterator](){return i;}});}catch(e){caught=e instanceof TypeError;}caught && closed===1 && escaped.get(key)===7",
    );
}

#[test]
fn valid_symbol_constructor_abort_skips_iterator_close_and_retains_prior_object_entries() {
    let mut realm = Realm::default();
    realm.eval("let key={},symbol=Symbol(),escaped,closed=0,flag=0,n=0,set=WeakMap.prototype.set;WeakMap.prototype.set=function(k,v){escaped=this;return set.call(this,k,v);};let i={next(){return {value:n++===0?[key,7]:[symbol,8]};},return(){closed++;return {};}}").unwrap();
    assert!(matches!(
        realm.eval(
            "try{new WeakMap({[Symbol.iterator](){return i;}});}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && closed===0 && escaped.get(key)===7"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn subclasses_bound_construction_and_nonstandard_prototypes_retain_the_private_slot() {
    check(
        "let key={};class C extends WeakMap{#initial=this.get(key);read(){return this.#initial;}}let m=new C([[key,7]]);m instanceof C && m instanceof WeakMap && m.read()===7 && m.get(key)===7",
    );
    check(
        "let key={},B=WeakMap.bind(null,[[key,7]]),m=new B();m instanceof WeakMap && m.get(key)===7",
    );
    check(
        "let key={},p={};function C(){}C.prototype=p;let m=Reflect.construct(WeakMap,[],C);Object.getPrototypeOf(m)===p && WeakMap.prototype.set.call(m,key,7)===m && WeakMap.prototype.get.call(m,key)===7",
    );
    check(
        "function C(){}C.prototype=1;Object.getPrototypeOf(Reflect.construct(WeakMap,[],C))===WeakMap.prototype",
    );
}

#[test]
fn reflection_and_descriptors_match_the_complete_noniterable_weakmap_surface() {
    let value=Realm::default().eval(r#"
        function flags(d){return (d.writable?'w':'-')+(d.enumerable?'e':'-')+(d.configurable?'c':'-');}
        let rows=[],global=Object.getOwnPropertyDescriptor(globalThis,'WeakMap');
        rows.push('constructor '+WeakMap.name+'/'+WeakMap.length+' parent='+(Object.getPrototypeOf(WeakMap)===Function.prototype));
        rows.push('global '+flags(global)+' prototype '+flags(Object.getOwnPropertyDescriptor(WeakMap,'prototype')));
        rows.push('constructor keys '+Reflect.ownKeys(WeakMap).join());
        rows.push('prototype parent='+(Object.getPrototypeOf(WeakMap.prototype)===Object.prototype)+' constructor='+(WeakMap.prototype.constructor===WeakMap));
        rows.push('prototype names '+Object.getOwnPropertyNames(WeakMap.prototype).join());
        rows.push('constructor property '+flags(Object.getOwnPropertyDescriptor(WeakMap.prototype,'constructor')));
        for(let name of ['delete','get','getOrInsert','getOrInsertComputed','has','set']){
            let f=WeakMap.prototype[name],d=Object.getOwnPropertyDescriptor(WeakMap.prototype,name);
            rows.push(name+' '+f.name+'/'+f.length+' '+flags(d)+' name='+flags(Object.getOwnPropertyDescriptor(f,'name'))+' length='+flags(Object.getOwnPropertyDescriptor(f,'length'))+' prototype='+Object.hasOwn(f,'prototype'));
        }
        let tag=Object.getOwnPropertyDescriptor(WeakMap.prototype,Symbol.toStringTag),m=new WeakMap();
        rows.push('tag '+tag.value+' '+flags(tag)+' symbols='+Object.getOwnPropertySymbols(WeakMap.prototype).length);
        rows.push('instance keys='+Reflect.ownKeys(m).length+' extensible='+Object.isExtensible(m)+' '+Object.prototype.toString.call(m));
        rows.push('absent size='+('size' in m)+' clear='+('clear' in m)+' iterator='+((Symbol.iterator) in m)+' species='+Object.hasOwn(WeakMap,Symbol.species));
        rows.join('\n')
    "#).unwrap();
    let Value::String(output) = value else {
        panic!("{value:?}")
    };
    insta::assert_snapshot!(output.to_utf8().unwrap(), @r###"
    constructor WeakMap/0 parent=true
    global w-c prototype ---
    constructor keys length,name,prototype
    prototype parent=true constructor=true
    prototype names constructor,delete,get,getOrInsert,getOrInsertComputed,has,set
    constructor property w-c
    delete delete/1 w-c name=--c length=--c prototype=false
    get get/1 w-c name=--c length=--c prototype=false
    getOrInsert getOrInsert/2 w-c name=--c length=--c prototype=false
    getOrInsertComputed getOrInsertComputed/2 w-c name=--c length=--c prototype=false
    has has/1 w-c name=--c length=--c prototype=false
    set set/2 w-c name=--c length=--c prototype=false
    tag WeakMap --c symbols=1
    instance keys=0 extensible=true [object WeakMap]
    absent size=false clear=false iterator=false species=false
    "###);
    for method in METHODS {
        type_error(&format!("new WeakMap.prototype.{method}()"));
    }
    type_error("let m=new WeakMap();for(let value of m){}");
}

#[test]
fn ordinary_object_integrity_does_not_freeze_entries_or_replace_the_internal_brand() {
    check(
        "let key={},other={},m=new WeakMap([[key,1]]);Object.freeze(m);m.set(key,2)===m && m.get(key)===2 && m.getOrInsert(other,3)===3 && m.delete(key) && Object.isFrozen(m) && Reflect.ownKeys(m).length===0",
    );
    check(
        "let key={},m=new WeakMap(),set=m.set,get=m.get,has=m.has;Object.setPrototypeOf(m,null);Object.freeze(m);set.call(m,key,7)===m && get.call(m,key)===7 && has.call(m,key)",
    );
    check(
        "Object.freeze(WeakMap);Object.freeze(WeakMap.prototype);let key={},m=new WeakMap([[key,7]]);m.get(key)===7 && m.delete(key)",
    );
}

#[test]
fn collection_keeps_active_conditional_values_and_reclaims_unreachable_back_edge_cycles() {
    let mut realm = Realm::default();
    realm
        .eval("let map=new WeakMap(),key={};map.set(key,{n:7,back:key})")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("map.get(key).n===7 && map.get(key).back===key"),
        Ok(Value::Boolean(true))
    );
    realm.eval("key=undefined").unwrap();
    assert!(realm.collect(usize::MAX).unwrap().reclaimed >= 2);
    assert_eq!(
        realm.eval(
            "let fresh={};map.set(fresh,9);map.has(fresh) && map.get(fresh)===9 && !map.has({})"
        ),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn conditional_values_activate_later_object_keys_in_either_map_allocation_order() {
    for declarations in [
        "let first=new WeakMap(),second=new WeakMap();",
        "let second=new WeakMap(),first=new WeakMap();",
    ] {
        let mut realm = Realm::default();
        realm.eval(&format!("{declarations}let root={{}};(()=>{{let middle={{}},value={{n:7}};first.set(root,middle);second.set(middle,value);}})()")).unwrap();
        realm.collect(usize::MAX).unwrap();
        assert_eq!(
            realm.eval("second.get(first.get(root)).n===7"),
            Ok(Value::Boolean(true))
        );
        realm.eval("root=undefined").unwrap();
        assert!(realm.collect(usize::MAX).unwrap().reclaimed >= 3);
    }
}

#[test]
fn failed_collection_retains_usable_object_entries_and_symbol_values_are_allowed() {
    let mut realm = Realm::default();
    realm.eval("let map=new WeakMap(),key={},other={};map.set(key,{n:7});map.set(other,Symbol('value'))").unwrap();
    assert!(realm.collect(0).is_err());
    assert_eq!(
        realm.eval("map.get(key).n===7 && typeof map.get(other)==='symbol'"),
        Ok(Value::Boolean(true))
    );
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("map.get(key).n===7 && typeof map.get(other)==='symbol'"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn explicit_work_abort_keeps_callback_effects_and_skips_language_cleanup() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5_000),
        ..Limits::default()
    });
    realm
        .eval("let map=new WeakMap(),key={},other={},flag=0,calls=0")
        .unwrap();
    assert!(matches!(realm.eval("try{map.getOrInsertComputed(key,()=>{calls++;map.set(other,7);while(true){}});}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit {..})));
    assert_eq!(
        realm.eval("flag===0 && calls===1 && !map.has(key) && map.get(other)===7"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn constructor_host_abort_does_not_close_the_iterator_or_enter_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(10_000),
        ..Limits::default()
    });
    realm
        .eval(
            "let closed=0,flag=0,i={next(){return {value:[{},7]};},return(){closed++;return {};}}",
        )
        .unwrap();
    assert!(matches!(
        realm.eval(
            "try{new WeakMap({[Symbol.iterator](){return i;}});}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && closed===0"),
        Ok(Value::Boolean(true))
    );
}
