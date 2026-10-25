//! FinalizationRegistry language semantics and explicit host cleanup (26.2).

use crate::common::REALM_ENTRIES;
use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, FinalizationCleanupOutcome, Limits, Realm, Value};

fn check(source: &str) {
    check_in(&mut Realm::default(), source);
}

fn check_in(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}

fn checkpoint_collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

fn completed(realm: &mut Realm) {
    let result = realm.run_finalization_cleanup_job();
    assert!(
        matches!(&result, Ok(FinalizationCleanupOutcome::Completed)),
        "{result:?}"
    );
}

fn no_job(realm: &mut Realm) {
    let result = realm.run_finalization_cleanup_job();
    assert!(
        matches!(&result, Ok(FinalizationCleanupOutcome::NoJob)),
        "{result:?}"
    );
}

fn reported(realm: &mut Realm) -> Error {
    match realm.run_finalization_cleanup_job() {
        Ok(FinalizationCleanupOutcome::Reported(error)) => error,
        result => panic!("expected a reported cleanup error: {result:?}"),
    }
}

#[test]
fn metadata_and_new_instance_surface_preserve_the_existing_intrinsics() {
    let mut realm = Realm::default();
    let Value::String(output) = realm.eval(r#"
        let C=FinalizationRegistry,P=C.prototype,R=P.register,U=P.unregister;
        function flags(d){return (d.writable?'W':'-')+(d.enumerable?'E':'-')+(d.configurable?'C':'-');}
        let instance=new C(()=>{}),tag=Object.getOwnPropertyDescriptor(P,Symbol.toStringTag);
        [
            'constructor:'+C.name+'/'+C.length,
            'prototype:'+flags(Object.getOwnPropertyDescriptor(C,'prototype')),
            'register:'+R.name+'/'+R.length+' '+flags(Object.getOwnPropertyDescriptor(P,'register')),
            'unregister:'+U.name+'/'+U.length+' '+flags(Object.getOwnPropertyDescriptor(P,'unregister')),
            'tag:'+tag.value+' '+flags(tag),
            'instance:'+Reflect.ownKeys(instance).length+' '+Object.isExtensible(instance)+' '+(Object.getPrototypeOf(instance)===P)
        ].join('\n')
    "#).unwrap() else { panic!("expected metadata string") };
    insta::assert_snapshot!(output.to_utf8().unwrap(), @"
    constructor:FinalizationRegistry/1
    prototype:---
    register:register/2 W-C
    unregister:unregister/1 W-C
    tag:FinalizationRegistry --C
    instance:0 true true
    ");
    check_in(
        &mut realm,
        "P.constructor===C && Object.getPrototypeOf(C)===Function.prototype && Object.getPrototypeOf(P)===Object.prototype && !('cleanupSome' in P) && !('gc' in instance) && !(Symbol.iterator in instance) && !(Symbol.species in C)",
    );
}

#[test]
fn constructor_requires_new_and_validates_callability_before_prototype_get() {
    for callback in [
        "undefined",
        "null",
        "false",
        "7",
        "1n",
        "'callback'",
        "Symbol()",
        "{}",
    ] {
        check(&format!(
            r#"
            let reads=0;function C(){{}}let N=C.bind(null);
            Object.defineProperty(N,'prototype',{{get(){{reads++;throw 7;}}}});
            let caught=false;try{{Reflect.construct(FinalizationRegistry,[{callback}],N);}}
            catch(e){{caught=e instanceof TypeError;}}caught && reads===0
        "#
        ));
    }
    check(
        r#"
        let reads=0,caught=false,callback={get [Symbol.toPrimitive](){reads++;throw 7;}};
        try{new FinalizationRegistry(callback);}catch(e){caught=e instanceof TypeError;}
        if(!caught || reads!==0)throw 1;
        caught=false;try{FinalizationRegistry(()=>{});}catch(e){caught=e instanceof TypeError;}
        caught
    "#,
    );
    check(
        r#"
        let marker={},reads=0,callback=()=>{};function C(){}let N=C.bind(null);
        Object.defineProperty(N,'prototype',{get(){reads++;throw marker;}});
        let caught=false;try{Reflect.construct(FinalizationRegistry,[callback],N);}catch(e){caught=e===marker;}
        caught && reads===1
    "#,
    );
}

#[test]
fn subclasses_and_intrinsic_prototype_fallback_initialize_only_real_instances() {
    check(
        r#"
        let C=FinalizationRegistry,P=C.prototype,register=P.register,unregister=P.unregister;
        class Derived extends C{constructor(callback){super(callback);this.marker=7;}}
        let instance=new Derived(()=>{}),target={},token={};
        if(!(instance instanceof Derived) || !(instance instanceof C) || instance.marker!==7 || Object.getPrototypeOf(instance)!==Derived.prototype)throw 1;
        if(register.call(instance,target,8,token)!==undefined || unregister.call(instance,token)!==true)throw 2;
        class Fake extends C{constructor(){return {};}}
        let caught=false;try{register.call(new Fake(),{},8);}catch(e){caught=e instanceof TypeError;}
        if(!caught)throw 3;
        this.FinalizationRegistry=function(){throw 4;};delete this.FinalizationRegistry;
        function N(){}
        for(let prototype of [undefined,null,false,7,'prototype',Symbol('prototype')]){
            N.prototype=prototype;let registry=Reflect.construct(C,[()=>{}],N);
            if(Object.getPrototypeOf(registry)!==P || register.call(registry,target,8,token)!==undefined || !unregister.call(registry,token))throw 5;
        }
        let custom=Object.create(null);N.prototype=custom;
        let registry=Reflect.construct(C,[()=>{}],N);
        Object.getPrototypeOf(registry)===custom && register.call(registry,target,8,token)===undefined && unregister.call(registry,token)
    "#,
    );
}

#[test]
fn proxy_new_targets_query_the_realm_only_after_nonobject_prototype_results() {
    check(
        r#"
        for(let bound of [false,true]){
            for(let objectResult of [false,true]){
                let armed=false,reads=0,hooks=0,pair,plain=function(){},chosen=Proxy.revocable({},{});
                chosen.revoke();let prototype=chosen.proxy,result=objectResult?prototype:0;
                if(!Object.getOwnPropertyDescriptor(plain,'prototype').writable)throw 1;
                pair=Proxy.revocable(plain,{
                    get(t,k,r){
                        if(!armed)return Reflect.get(t,k,r);
                        if(!bound && k==='prototype'){reads++;pair.revoke();return result;}
                        hooks++;throw 2;
                    },
                    getPrototypeOf(t){if(armed){hooks++;throw 3;}return Reflect.getPrototypeOf(t);},
                    construct(){hooks++;throw 4;},apply(){hooks++;throw 5;}
                });
                let N=bound?pair.proxy.bind(null).bind(null):pair.proxy;
                if(bound)Object.defineProperty(N,'prototype',{get(){reads++;pair.revoke();return result;}});
                armed=true;let caught=false,registry;
                try{registry=Reflect.construct(FinalizationRegistry,[()=>{}],N);}
                catch(e){if(!(e instanceof TypeError))throw e;caught=true;}
                if(reads!==1 || hooks!==0 || caught===objectResult)throw 6;
                if(objectResult){
                    let target={},token={};
                    if(Object.getPrototypeOf(registry)!==prototype || FinalizationRegistry.prototype.register.call(registry,target,7,token)!==undefined || !FinalizationRegistry.prototype.unregister.call(registry,token))throw 7;
                }
            }
        }
        true
    "#,
    );
}

#[test]
fn methods_require_own_slots_and_never_unwrap_proxy_receivers() {
    check(
        r#"
        let registry=new FinalizationRegistry(()=>{}),hits=0,handler={
            get(){hits++;throw 7;},getPrototypeOf(){hits++;throw 8;},ownKeys(){hits++;throw 9;}
        },pair=Proxy.revocable(registry,handler);
        pair.revoke();
        let receivers=[undefined,null,false,7,1n,'x',Symbol(),{},FinalizationRegistry.prototype,
            Object.create(registry),new WeakMap(),{[Symbol.toStringTag]:'FinalizationRegistry'},
            new Proxy(registry,handler),pair.proxy];
        for(let receiver of receivers){
            for(let name of ['register','unregister']){
                let caught=false;
                try{Reflect.apply(FinalizationRegistry.prototype[name],receiver,[{},7,{}]);}
                catch(e){caught=e instanceof TypeError;}
                if(!caught)throw 1;
            }
        }
        for(let method of [FinalizationRegistry.prototype.register,FinalizationRegistry.prototype.unregister]){
            let caught=false;try{Reflect.construct(method,[]);}catch(e){caught=e instanceof TypeError;}
            if(!caught)throw 2;
        }
        let fake=Object.create(FinalizationRegistry.prototype);
        hits===0 && Object.prototype.toString.call(fake)==='[object FinalizationRegistry]'
    "#,
    );
}

#[test]
fn target_token_categories_and_samevalue_rules_do_not_execute_coercion_hooks() {
    check(
        r#"
        let hits=0,registry=new FinalizationRegistry(()=>{}),registered=Symbol.for('fr-invalid');
        Symbol.keyFor=function(){hits++;throw 7;};
        let hostile={get [Symbol.toPrimitive](){hits++;throw 8;},valueOf(){hits++;throw 9;},toString(){hits++;throw 10;}};
        let pair=Proxy.revocable(function(){},{get(){hits++;throw 11;},getPrototypeOf(){hits++;throw 12;}});pair.revoke();
        let valid=[{},Object.create(null),[],function(){},()=>{},function(){}.bind(null),hostile,pair.proxy,
            Object(registered),Object(Symbol()),new Map(),new Set(),new WeakMap(),new WeakSet(),
            new ArrayBuffer(0),new DataView(new ArrayBuffer(0)),new Uint8Array(0),new BigInt64Array(0),
            Symbol(),Symbol('same'),Symbol('same'),Symbol.iterator];
        for(let target of valid){
            let token={};if(registry.register(target,undefined,token)!==undefined || !registry.unregister(token))throw 1;
            if(registry.register({},registered,target)!==undefined || !registry.unregister(target))throw 2;
            let caught=false;try{registry.register(target,target);}catch(e){caught=e instanceof TypeError;}
            if(!caught)throw 3;
        }
        for(let invalid of [undefined,null,false,7,NaN,1n,'x',registered]){
            let a=false,b=false,c=false;
            try{registry.register(invalid,8);}catch(e){a=e instanceof TypeError;}
            if(invalid!==undefined){try{registry.register({},8,invalid);}catch(e){b=e instanceof TypeError;}}
            else b=registry.register({},8,undefined)===undefined;
            try{registry.unregister(invalid);}catch(e){c=e instanceof TypeError;}
            if(!a || !b || !c)throw 4;
        }
        registry.register({},8);hits===0
    "#,
    );
}

#[test]
fn duplicate_cells_and_unregister_all_work_before_and_after_target_clearing() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let log=[],registry=new FinalizationRegistry(v=>log.push(v)),target={},token={},sameA=Symbol('same'),sameB=Symbol('same');
        registry.register(target,1,token);registry.register(target,2,token);registry.register(target,3,sameA);
        registry.register(target,4,sameB);registry.register({},5,token);registry.register({},6,sameA);
        if(!registry.unregister(token) || registry.unregister(token))throw 1;
        if(!registry.unregister(sameB) || registry.unregister(sameB))throw 2;
        let rejected=false;try{registry.register(target,target,sameA);}catch(e){rejected=e instanceof TypeError;}
        if(!rejected)throw 3;target=undefined;
    "#).unwrap();
    checkpoint_collect(&mut realm);
    check_in(
        &mut realm,
        "registry.unregister(sameA) && !registry.unregister(sameA) && log.length===0",
    );
    let result = realm.run_finalization_cleanup_job();
    assert!(
        matches!(
            &result,
            Ok(FinalizationCleanupOutcome::NoJob | FinalizationCleanupOutcome::Completed)
        ),
        "{result:?}"
    );
    no_job(&mut realm);
    check_in(&mut realm, "log.length===0");
}

#[test]
fn held_values_are_delivered_verbatim_as_one_argument_with_ordinary_call_semantics() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let pair=Proxy.revocable({},{});pair.revoke();
        let held=[undefined,null,true,NaN,-0,1n,Symbol('held'),Symbol.for('held'),pair.proxy,{},function(){}],log=[];
        let registry=new FinalizationRegistry(function(value){'use strict';if(this!==undefined || arguments.length!==1)throw 1;log.push(value);return 99;});
        for(let value of held)registry.register({},value);
    "#).unwrap();
    checkpoint_collect(&mut realm);
    completed(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "log.length===held.length && held.every(value=>log.some(item=>Object.is(item,value)))",
    );
}

#[test]
fn registration_never_keeps_its_target_even_when_the_target_is_its_token() {
    let mut realm = Realm::default();
    let object = realm
        .eval(
            r#"
        let log=[],registry=new FinalizationRegistry(v=>log.push(v));
        (()=>{let target={};registry.register(target,1,target);return target;})()
    "#,
        )
        .unwrap();
    let Value::Object(handle) = object else {
        panic!("expected object target")
    };
    no_job(&mut realm);
    // No WeakRef or checkpoint was involved: register must not add a kept root.
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(&handle).is_err());
    completed(&mut realm);
    check_in(&mut realm, "log.length===1 && log[0]===1");
    realm
        .eval("(()=>{let target=Symbol('target-token');registry.register(target,2,target);})()")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    completed(&mut realm);
    no_job(&mut realm);
    check_in(&mut realm, "log.length===2 && log[1]===2");
    let rejected_target=realm.eval(r#"
        (()=>{
            let target={},caught=0;registry.register(target,3);
            try{registry.register(target,target,{});}catch(e){if(!(e instanceof TypeError))throw e;caught++;}
            try{registry.register(target,99,Symbol.for('invalid-token'));}catch(e){if(!(e instanceof TypeError))throw e;caught++;}
            if(caught!==2)throw 1;return target;
        })()
    "#).unwrap();
    let Value::Object(rejected_handle) = rejected_target else {
        panic!("expected object target")
    };
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(&rejected_handle).is_err());
    completed(&mut realm);
    no_job(&mut realm);
    check_in(&mut realm, "log.length===3 && log[2]===3");
}

#[test]
fn dying_tokens_do_not_unregister_or_schedule_their_live_targets() {
    for token in ["{}", "Symbol('token')"] {
        let mut realm = Realm::default();
        realm.eval(&format!(r#"
            let log=[],target={{}},registry=new FinalizationRegistry(v=>log.push(v)),tokenRef;
            (()=>{{let token={token};tokenRef=new WeakRef(token);registry.register(target,7,token);}})()
        "#)).unwrap();
        checkpoint_collect(&mut realm);
        check_in(&mut realm, "tokenRef.deref()===undefined && log.length===0");
        no_job(&mut realm);
        realm.eval("target=undefined").unwrap();
        realm.collect(usize::MAX).unwrap();
        completed(&mut realm);
        no_job(&mut realm);
        check_in(&mut realm, "log.length===1 && log[0]===7");
    }
}

#[test]
fn registry_held_graphs_keep_targets_live_until_the_real_graph_is_broken() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let log=[],registry=new FinalizationRegistry(value=>log.push(value)),targetRef,heldRef;
        (()=>{let target={},held={target};targetRef=new WeakRef(target);heldRef=new WeakRef(held);registry.register(target,held);})()
    "#).unwrap();
    checkpoint_collect(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "targetRef.deref()===heldRef.deref().target && log.length===0",
    );
    realm.eval("heldRef.deref().target=undefined").unwrap();
    checkpoint_collect(&mut realm);
    check_in(
        &mut realm,
        "targetRef.deref()===undefined && heldRef.deref()!==undefined",
    );
    completed(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "log.length===1 && log[0]===heldRef.deref() && log[0].target===undefined",
    );
}

#[test]
fn the_stored_callback_and_its_environment_are_strong_registry_edges() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let log=[],registry,callbackRef,targetRef;
        (()=>{
            let target={},box={target};
            let callback=function(value){if(box.target!==undefined)throw 1;log.push(value);};
            callback.release=function(){box.target=undefined;};
            callbackRef=new WeakRef(callback);targetRef=new WeakRef(target);
            registry=new FinalizationRegistry(callback);registry.register(target,7);target=undefined;
        })()
    "#).unwrap();
    checkpoint_collect(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "typeof callbackRef.deref()==='function' && targetRef.deref()!==undefined",
    );
    realm.eval("callbackRef.deref().release()").unwrap();
    checkpoint_collect(&mut realm);
    completed(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "targetRef.deref()===undefined && log.length===1 && log[0]===7",
    );
}

#[test]
fn held_symbols_are_live_without_any_symbol_keyed_weakmap_history() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let log=[],registry=new FinalizationRegistry(value=>log.push(value)),heldRef,target={};
        (()=>{let held=Symbol('held');heldRef=new WeakRef(held);registry.register(target,held);})()
    "#,
        )
        .unwrap();
    checkpoint_collect(&mut realm);
    no_job(&mut realm);
    check_in(&mut realm, "typeof heldRef.deref()==='symbol'");
    realm.eval("target=undefined").unwrap();
    checkpoint_collect(&mut realm);
    completed(&mut realm);
    check_in(
        &mut realm,
        "log.length===1 && typeof log[0]==='symbol' && log[0]===heldRef.deref()",
    );
    realm.eval("log=[]").unwrap();
    checkpoint_collect(&mut realm);
    check_in(&mut realm, "heldRef.deref()===undefined");
    no_job(&mut realm);
}

#[test]
fn dead_registry_callback_and_held_cycles_are_not_rescued_by_their_targets() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let target={},references,unrelated=new FinalizationRegistry(()=>{});
        (()=>{
            let registry,held={};registry=new FinalizationRegistry(()=>registry);
            held.owner=registry;registry.register(target,held);
            references=[new WeakRef(registry),new WeakRef(held)];
        })()
    "#,
        )
        .unwrap();
    checkpoint_collect(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "references.every(reference=>reference.deref()===undefined) && typeof target==='object' && unrelated instanceof FinalizationRegistry",
    );
}

#[test]
fn an_actual_queued_job_keeps_its_registry_callback_and_held_value_alive() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let log=[],registry,registryRef,heldRef;
        (()=>{
            let held={n:7};registry=new FinalizationRegistry(value=>log.push(value));
            registry.register({},held);registryRef=new WeakRef(registry);heldRef=new WeakRef(held);
        })()
    "#,
        )
        .unwrap();
    checkpoint_collect(&mut realm);
    realm.eval("registry=undefined").unwrap();
    checkpoint_collect(&mut realm);
    check_in(
        &mut realm,
        "registryRef.deref()!==undefined && heldRef.deref()!==undefined && log.length===0",
    );
    completed(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "log.length===1 && log[0]===heldRef.deref() && log[0].n===7",
    );
    checkpoint_collect(&mut realm);
    check_in(&mut realm, "registryRef.deref()===undefined");
}

#[test]
fn independent_host_roots_prevent_cleanup_until_the_last_root_is_dropped() {
    let mut realm = Realm::default();
    let target=realm.eval("let log=[],registry=new FinalizationRegistry(v=>log.push(v));(()=>{let target={n:7};registry.register(target,8);return target;})()").unwrap();
    let root = realm.root_value(target.clone(), usize::MAX).unwrap();
    let clone = root.clone();
    drop(root);
    checkpoint_collect(&mut realm);
    no_job(&mut realm);
    assert_eq!(
        realm.read_property(&target, &JsString::from("n")),
        Ok(Value::Number(7.0))
    );
    assert_eq!(clone.value(), &target);
    drop(clone);
    realm.collect(usize::MAX).unwrap();
    completed(&mut realm);
    no_job(&mut realm);
    check_in(&mut realm, "log.length===1 && log[0]===8");
}

#[test]
fn releasing_a_held_graph_only_enables_another_registry_on_a_later_collection() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let log=[],first=new FinalizationRegistry(value=>log.push('first')),
            second=new FinalizationRegistry(value=>log.push('second')),secondRef;
        (()=>{let target={},held={target};secondRef=new WeakRef(target);second.register(target,7);first.register({},held);})()
    "#).unwrap();
    checkpoint_collect(&mut realm);
    completed(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "log.length===1 && log[0]==='first' && secondRef.deref()!==undefined",
    );
    checkpoint_collect(&mut realm);
    completed(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "log.join(',')==='first,second' && secondRef.deref()===undefined",
    );
}

#[test]
fn weakrefs_and_registrations_agree_on_mixed_ephemeron_cycle_death() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let log=[],registry=new FinalizationRegistry(value=>log.push(value)),first=new WeakMap(),second=new WeakMap(),objectRef,symbolRef;
        (()=>{let object={},symbol=Symbol('cycle');first.set(object,symbol);second.set(symbol,{back:object});
            objectRef=new WeakRef(object);symbolRef=new WeakRef(symbol);registry.register(object,7);registry.register(symbol,8);})()
    "#).unwrap();
    realm.collect(usize::MAX).unwrap();
    no_job(&mut realm);
    check_in(
        &mut realm,
        "first.get(objectRef.deref())===symbolRef.deref() && second.get(symbolRef.deref()).back===objectRef.deref()",
    );
    checkpoint_collect(&mut realm);
    check_in(
        &mut realm,
        "objectRef.deref()===undefined && symbolRef.deref()===undefined",
    );
    completed(&mut realm);
    no_job(&mut realm);
    check_in(&mut realm, "log.slice().sort((a,b)=>a-b).join(',')==='7,8'");
}

#[test]
fn empty_cleanup_is_inert_and_collect_or_checkpoint_never_runs_javascript() {
    let mut realm = Realm::default();
    no_job(&mut realm);
    realm.clear_kept_objects().unwrap();
    no_job(&mut realm);
    assert_eq!(realm.collect(usize::MAX).unwrap().live, 0);
    realm.eval("0").unwrap();
    no_job(&mut realm);
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    realm.eval("let calls=0,registry=new FinalizationRegistry(()=>{calls++;throw 7;});registry.register({},8)").unwrap();
    no_job(&mut realm);
    realm.clear_kept_objects().unwrap();
    check_in(&mut realm, "calls===0");
    realm.collect(usize::MAX).unwrap();
    check_in(&mut realm, "calls===0");
    realm.clear_kept_objects().unwrap();
    check_in(&mut realm, "calls===0");
    assert_eq!(reported(&mut realm), Error::Thrown(Value::Number(7.0)));
    no_job(&mut realm);
    check_in(&mut realm, "calls===1");
}

#[test]
fn ordinary_evaluations_and_host_property_reads_do_not_end_the_kept_sequence() {
    let mut realm = Realm::default();
    let observer = realm
        .eval(
            r#"
        let calls=0,reference,registry=new FinalizationRegistry(()=>calls++);
        (()=>{let target={};reference=new WeakRef(target);registry.register(target,7);})()
        ;({get value(){return 8;}})
    "#,
        )
        .unwrap();
    let observer_root = realm.root_value(observer.clone(), usize::MAX).unwrap();
    for _ in 0..2 {
        realm.eval("({unrelated:7})").unwrap();
        assert_eq!(
            realm.read_property(&observer, &JsString::from("value")),
            Ok(Value::Number(8.0))
        );
        realm.collect(usize::MAX).unwrap();
        no_job(&mut realm);
        check_in(&mut realm, "calls===0 && reference.deref()!==undefined");
    }
    drop(observer_root);
    checkpoint_collect(&mut realm);
    check_in(&mut realm, "calls===0 && reference.deref()===undefined");
    completed(&mut realm);
    no_job(&mut realm);
    check_in(&mut realm, "calls===1");
}

#[test]
fn prototype_getter_mutation_does_not_replace_the_original_job_callback() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let log=[],callback=value=>log.push('original:'+value),prototype=FinalizationRegistry.prototype;
        function C(){}let N=C.bind(null);
        Object.defineProperty(N,'prototype',{get(){callback=value=>log.push('replacement:'+value);return prototype;}});
        let registry=Reflect.construct(FinalizationRegistry,[callback],N);registry.register({},7);
    "#).unwrap();
    checkpoint_collect(&mut realm);
    completed(&mut realm);
    no_job(&mut realm);
    check_in(&mut realm, "log.join(',')==='original:7'");
}

#[test]
fn job_callback_proxy_call_uses_undefined_this_and_ignores_public_call_helpers() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reads=0,log=[],correct=false;
        let callback=new Proxy(function(){throw 1;},{
            get(){reads++;throw 2;},getPrototypeOf(){reads++;throw 3;},
            apply(target,receiver,args){correct=receiver===undefined && args.length===1;log.push(args[0]);return 99;}
        });
        let registry=new FinalizationRegistry(callback);registry.register({},7);
    "#).unwrap();
    checkpoint_collect(&mut realm);
    completed(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "reads===0 && correct && log.length===1 && log[0]===7",
    );
}

#[test]
fn a_revoked_callable_callback_is_accepted_at_construction_and_reported_on_call() {
    let mut realm = Realm::default();
    realm.eval("let pair=Proxy.revocable(function(){},{});pair.revoke();let registry=new FinalizationRegistry(pair.proxy);registry.register({},7)").unwrap();
    checkpoint_collect(&mut realm);
    assert!(matches!(
        reported(&mut realm),
        Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        }
    ));
    no_job(&mut realm);
    assert_eq!(realm.eval("2+3"), Ok(Value::Number(5.0)));
}

#[test]
fn selected_cell_is_removed_before_callback_side_unregister() {
    let mut realm = Realm::default();
    realm.eval("let log=[],token={},registry=new FinalizationRegistry(value=>log.push([value,registry.unregister(token)]));registry.register({},7,token)").unwrap();
    checkpoint_collect(&mut realm);
    completed(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "log.length===1 && log[0][0]===7 && log[0][1]===false && !registry.unregister(token)",
    );
}

#[test]
fn callback_mutation_reselects_current_cells_and_new_live_targets_wait_for_collection() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let log=[],token={},newTarget={},newToken={},removed=false,registry;
        let register=FinalizationRegistry.prototype.register,unregister=FinalizationRegistry.prototype.unregister;
        registry=new FinalizationRegistry(value=>{
            log.push(value);
            if(log.length===1){
                removed=unregister.call(registry,token);register.call(registry,newTarget,99,newToken);
                registry.callback=()=>{throw 7;};Object.setPrototypeOf(registry,null);Object.freeze(registry);
            }
        });
        registry.register({},1,token);registry.register({},2,token);registry.register({},3,token);
    "#).unwrap();
    checkpoint_collect(&mut realm);
    completed(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "removed && log.length===1 && log[0]>=1 && log[0]<=3 && Object.isFrozen(registry)",
    );
    realm.eval("newTarget=undefined").unwrap();
    realm.collect(usize::MAX).unwrap();
    completed(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "log.length===2 && log[1]===99 && !unregister.call(registry,newToken)",
    );
}

#[test]
fn repeated_collections_coalesce_and_each_host_call_runs_only_one_registry_job() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let log=[],first=new FinalizationRegistry(()=>log.push('first')),second=new FinalizationRegistry(()=>log.push('second'));
        first.register({},1);first.register({},2);second.register({},3);second.register({},4);
    "#).unwrap();
    checkpoint_collect(&mut realm);
    realm.collect(usize::MAX).unwrap();
    realm.collect(usize::MAX).unwrap();
    completed(&mut realm);
    check_in(&mut realm, "log.length===2 && log[0]===log[1]");
    completed(&mut realm);
    check_in(
        &mut realm,
        "log.length===4 && log[2]===log[3] && log[0]!==log[2]",
    );
    no_job(&mut realm);
}

#[test]
fn first_throw_is_reported_without_retrying_the_removed_cell_or_losing_remaining_capture() {
    let mut realm = Realm::default();
    let marker=realm.eval(r#"
        let attempts=[],marker={},registry=new FinalizationRegistry(value=>{attempts.push(value);throw marker;});
        registry.register({},1);registry.register({},2);registry.register({},3);marker
    "#).unwrap();
    checkpoint_collect(&mut realm);
    realm.eval("registry=undefined").unwrap();
    for expected in 1..=3 {
        assert_eq!(reported(&mut realm), Error::Thrown(marker.clone()));
        check_in(
            &mut realm,
            &format!("attempts.length==={expected} && new Set(attempts).size==={expected}"),
        );
        // The preexisting capture must survive between abrupt invocations.
        realm.collect(usize::MAX).unwrap();
    }
    no_job(&mut realm);
    check_in(
        &mut realm,
        "attempts.slice().sort((a,b)=>a-b).join(',')==='1,2,3'",
    );
    assert_eq!(realm.eval("2+3"), Ok(Value::Number(5.0)));
}

#[test]
fn weakref_symbols_stay_kept_between_callbacks_and_after_normal_or_abrupt_job_exit() {
    for abrupt in [false, true] {
        let mut realm = Realm::default();
        realm
            .eval(&format!(
                r#"
            let calls=0,reference,observed=false,prior=new WeakRef({{n:7}});
            let registry=new FinalizationRegistry(value=>{{
                calls++;
                if(calls===1){{let target=Symbol('job-kept');reference=new WeakRef(target);}}
                else observed=typeof reference.deref()==='symbol';
                if({abrupt})throw 8;
            }});
            registry.register({{}},1);registry.register({{}},2);
        "#
            ))
            .unwrap();
        // Only prior's target is kept; unrelated registration targets may die.
        realm.collect(usize::MAX).unwrap();
        if abrupt {
            assert_eq!(reported(&mut realm), Error::Thrown(Value::Number(8.0)));
            realm.collect(usize::MAX).unwrap();
            check_in(
                &mut realm,
                "calls===1 && typeof reference.deref()==='symbol' && prior.deref().n===7",
            );
            assert_eq!(reported(&mut realm), Error::Thrown(Value::Number(8.0)));
        } else {
            completed(&mut realm);
        }
        no_job(&mut realm);
        realm.collect(usize::MAX).unwrap();
        check_in(
            &mut realm,
            "calls===2 && observed && typeof reference.deref()==='symbol' && prior.deref().n===7",
        );
        checkpoint_collect(&mut realm);
        check_in(
            &mut realm,
            "reference.deref()===undefined && prior.deref()===undefined",
        );
    }
}

#[test]
fn opted_in_callback_work_abort_is_reported_and_does_not_retry_or_clear_kept_roots() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5_000),
        ..Limits::default()
    });
    realm.eval("let attempts=[],reference,flag=0").unwrap();
    realm
        .eval(
            r#"
        let registry=new FinalizationRegistry(value=>{
            attempts.push(value);
            if(attempts.length===1){
                reference=new WeakRef(Symbol('abrupt-kept'));
                try{while(true){}}catch{flag=1;}finally{flag=2;}
            }
        });
    "#,
        )
        .unwrap();
    realm
        .eval("registry.register({},1);registry.register({},2)")
        .unwrap();
    checkpoint_collect(&mut realm);
    assert!(matches!(reported(&mut realm), Error::Limit { .. }));
    check_in(
        &mut realm,
        "attempts.length===1 && flag===0 && typeof reference.deref()==='symbol'",
    );
    completed(&mut realm);
    no_job(&mut realm);
    check_in(
        &mut realm,
        "attempts.slice().sort((a,b)=>a-b).join(',')==='1,2' && typeof reference.deref()==='symbol'",
    );
    assert_eq!(realm.eval("2+3"), Ok(Value::Number(5.0)));
    checkpoint_collect(&mut realm);
    check_in(&mut realm, "reference.deref()===undefined");
}

#[test]
fn callbacks_share_one_opted_in_job_allowance_instead_of_resetting_it_each_call() {
    fn fixture(cells: usize) -> Realm {
        let mut realm = Realm::new(Limits {
            max_steps: Some(5_000),
            ..Limits::default()
        });
        // Use the member lookup during setup so the fixed job allowance does not
        // depend on repeated global binding searches as the intrinsic graph grows.
        realm.eval("let finished=0,registry=new globalThis.FinalizationRegistry(()=>{let total=0;for(let i=0;i<128;i++)total+=i;if(total!==8128)throw 7;finished++;})").unwrap();
        // Small ordinary evaluations prepare the fixture under the same limit.
        // They do not reset the allowance inside the cleanup job under test.
        for offset in (0..cells).step_by(8) {
            realm
                .eval(&format!(
                    "for(let i={offset};i<{};i++)registry.register({{}},i)",
                    (offset + 8).min(cells)
                ))
                .unwrap();
        }
        checkpoint_collect(&mut realm);
        realm
    }
    let mut single = fixture(1);
    completed(&mut single);
    check_in(&mut single, "finished===1");
    let mut grouped = fixture(64);
    assert!(matches!(reported(&mut grouped), Error::Limit { .. }));
    check_in(&mut grouped, "finished>=1 && finished<64");
    assert_eq!(grouped.eval("2+3"), Ok(Value::Number(5.0)));
}
