//! WeakRef construction, real-slot branding, and explicit host checkpoints (26.1).

use crate::common::REALM_ENTRIES;
use spite_core::{JsString, WellKnownSymbol};
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

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
fn construction_requires_new_and_rejects_invalid_targets_without_coercion() {
    type_error("WeakRef({})");
    type_error("new WeakRef()");
    for target in [
        "undefined",
        "null",
        "false",
        "true",
        "0",
        "-0",
        "NaN",
        "Infinity",
        "1n",
        "''",
        "'target'",
        "Symbol.for('weak-ref-registered')",
    ] {
        check(&format!(
            "let hits=0,registered=Symbol.for('weak-ref-registered');Symbol.keyFor=function(){{hits++;throw 7;}};let caught=false;try{{new WeakRef({target});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
        ));
    }
    check(
        "let hits=0,caught=false;try{WeakRef((hits++,{}),(hits++,7));}catch(e){caught=e instanceof TypeError;}caught && hits===2",
    );
}

#[test]
fn all_supported_object_kinds_and_nonregistered_symbols_preserve_identity() {
    check(
        r#"
        let registered=Symbol.for('weak-ref-boxed'),targets=[
            {},Object.create(null),[],function(){},()=>7,class C{},function(){}.bind(null),
            new Boolean(false),new Number(0),new String(''),Object(1n),
            Object(registered),Object(Symbol('boxed')),new Date(0),/x/,
            new Map(),new Set(),new WeakMap(),new WeakSet(),
            new ArrayBuffer(0),new DataView(new ArrayBuffer(0)),
            new Uint8Array(0),new BigInt64Array(0),new Error('x'),
            [1].values(),(function(){return arguments;})(),this,
            Symbol(),Symbol('same'),Symbol('same')
        ];
        let hostile={get [Symbol.toPrimitive](){throw 7;},valueOf(){throw 8;},toString(){throw 9;}};
        targets.push(hostile,new WeakRef(hostile));
        for(let target of targets){
            let a=new WeakRef(target),b=new WeakRef(target,()=>{throw 10;});
            if(a===b || a.deref()!==target || b.deref()!==target || a.deref()!==b.deref())throw 11;
        }
        targets[targets.length-4]!==targets[targets.length-3]
        "#,
    );
    for symbol in WellKnownSymbol::ALL {
        check(&format!(
            "let target=Symbol.{},reference=new WeakRef(target);reference.deref()===target",
            symbol.name()
        ));
    }
}

#[test]
fn target_validation_precedes_new_target_prototype_get() {
    for target in ["undefined", "7", "Symbol.for('weak-ref-invalid')"] {
        check(&format!(
            "let reads=0;function C(){{}}let N=C.bind(null);Object.defineProperty(N,'prototype',{{get(){{reads++;throw 7;}}}});let caught=false;try{{Reflect.construct(WeakRef,[{target}],N);}}catch(e){{caught=e instanceof TypeError;}}caught && reads===0"
        ));
    }
    check(
        r#"
        let log='',target={},prototype=Object.create(WeakRef.prototype);
        function C(){}let N=C.bind(null);
        Object.defineProperty(N,'prototype',{get(){log+='p';return prototype;}});
        let reference=Reflect.construct(WeakRef,[(log+='t',target)],N);
        log==='tp' && Object.getPrototypeOf(reference)===prototype && reference.deref()===target
        "#,
    );
    type_error("Reflect.construct(WeakRef,[{}],()=>{})");
}

#[test]
fn prototype_getter_abrupts_propagate_and_cannot_change_the_original_target() {
    check(
        r#"
        let target={},original=target,other={},marker={},calls=0;
        function C(){}let N=C.bind(null);
        Object.defineProperty(N,'prototype',{get(){calls++;target=other;throw marker;}});
        let caught=false;try{Reflect.construct(WeakRef,[target],N);}catch(e){caught=e===marker;}
        if(!caught || calls!==1 || target!==other)throw 1;
        Object.defineProperty(C,'prototype',{value:WeakRef.prototype});
        let M=C.bind(null);
        Object.defineProperty(M,'prototype',{get(){calls++;target=other;return WeakRef.prototype;}});
        target=original;
        let reference=Reflect.construct(WeakRef,[target],M);
        calls===2 && target===other && reference.deref()===original
        "#,
    );
}

#[test]
fn subclassing_initializes_only_instances_that_call_the_constructor() {
    check(
        r#"
        let target={};class Derived extends WeakRef{constructor(value){super(value);this.marker=7;}}
        let reference=new Derived(target);
        reference instanceof Derived && reference instanceof WeakRef &&
        Object.getPrototypeOf(reference)===Derived.prototype && reference.marker===7 &&
        reference.deref()===target && WeakRef.prototype.deref.call(reference)===target
        "#,
    );
    type_error(
        "class Derived extends WeakRef{constructor(){return {};}}WeakRef.prototype.deref.call(new Derived())",
    );
    type_error("class Derived extends WeakRef{}new Derived(7)");
}

#[test]
fn prototype_fallback_uses_the_intrinsic_after_global_replacement_and_deletion() {
    check(
        r#"
        let C=WeakRef,prototype=C.prototype,deref=prototype.deref,target={};
        this.WeakRef=function(){throw 7;};delete this.WeakRef;
        function N(){}
        for(let value of [undefined,null,false,7,'prototype',Symbol('prototype')]){
            N.prototype=value;
            let reference=Reflect.construct(C,[target],N);
            if(Object.getPrototypeOf(reference)!==prototype || deref.call(reference)!==target)throw 8;
        }
        let custom=Object.create(null);N.prototype=custom;
        let reference=Reflect.construct(C,[target],N);
        typeof WeakRef==='undefined' && Object.getPrototypeOf(reference)===custom && deref.call(reference)===target
        "#,
    );
}

#[test]
fn proxy_targets_are_valid_even_when_revoked_and_are_not_inspected() {
    check(
        r#"
        let hits=0,handler={get(){hits++;throw 7;},getPrototypeOf(){hits++;throw 8;},
            ownKeys(){hits++;throw 9;},isExtensible(){hits++;throw 10;}},target={};
        let proxy=new Proxy(target,handler),reference=new WeakRef(proxy);
        let revoked=Proxy.revocable(function(){},handler),other=new WeakRef(revoked.proxy);
        revoked.revoke();
        reference.deref()===proxy && other.deref()===revoked.proxy && hits===0
        "#,
    );
}

#[test]
fn callable_proxy_new_targets_observe_only_the_required_prototype_lookup() {
    check(
        r#"
        let log='',target={},prototype=Object.create(WeakRef.prototype);
        let N=new Proxy(function(){},{get(t,k,r){log+=String(k)+',';if(k==='prototype')return prototype;throw 7;},
            construct(){throw 8;},apply(){throw 9;}});
        let reference=Reflect.construct(WeakRef,[target],N);
        log==='prototype,' && Object.getPrototypeOf(reference)===prototype && reference.deref()===target
        "#,
    );
    type_error(
        "let pair=Proxy.revocable(function(){},{});pair.revoke();Reflect.construct(WeakRef,[{}],pair.proxy)",
    );
    // GetPrototypeFromConstructor (10.1.14) queries the function Realm only
    // for a nonobject result, after the observable prototype Get has finished.
    check(
        r#"
        for(let bound of [false,true]){
            for(let objectResult of [false,true]){
                let armed=false,hooks=0,reads=0,pair,target={},prototype=Object.create(WeakRef.prototype);
                let result=objectResult?prototype:0,plain=function(){};
                if(!Object.getOwnPropertyDescriptor(plain,'prototype').writable)throw 1;
                let handler={
                    get(t,k,r){
                        if(!armed)return Reflect.get(t,k,r);
                        if(!bound && k==='prototype'){reads++;pair.revoke();return result;}
                        hooks++;throw 2;
                    },
                    getPrototypeOf(t){if(armed){hooks++;throw 3;}return Reflect.getPrototypeOf(t);},
                    construct(){hooks++;throw 4;},apply(){hooks++;throw 5;}
                };
                pair=Proxy.revocable(plain,handler);
                let N=bound?pair.proxy.bind(null).bind(null):pair.proxy;
                if(bound)Object.defineProperty(N,'prototype',{get(){reads++;pair.revoke();return result;}});
                armed=true;let caught=false,reference;
                try{reference=Reflect.construct(WeakRef,[target],N);}
                catch(e){if(!(e instanceof TypeError))throw e;caught=true;}
                if(reads!==1 || hooks!==0 || caught===objectResult)throw 6;
                if(objectResult && (Object.getPrototypeOf(reference)!==prototype || WeakRef.prototype.deref.call(reference)!==target))throw 7;
            }
        }
        true
        "#,
    );
}

#[test]
fn deref_requires_an_own_real_slot_and_does_not_unwrap_proxy_receivers() {
    for receiver in [
        "undefined",
        "null",
        "false",
        "7",
        "1n",
        "'x'",
        "Symbol()",
        "{}",
        "WeakRef.prototype",
        "Object.create(new WeakRef({}))",
        "new WeakMap()",
        "{[Symbol.toStringTag]:'WeakRef',target:{}}",
    ] {
        type_error(&format!("WeakRef.prototype.deref.call({receiver})"));
    }
    check(
        r#"
        let reference=new WeakRef({}),hits=0,handler={get(){hits++;throw 7;},getPrototypeOf(){hits++;throw 8;}};
        let revoked=Proxy.revocable(reference,handler),receivers=[new Proxy(reference,handler),revoked.proxy];
        revoked.revoke();let caught=0;
        for(let receiver of receivers){try{Reflect.apply(WeakRef.prototype.deref,receiver,[]);}catch(e){if(!(e instanceof TypeError))throw e;caught++;}}
        caught===2 && hits===0
        "#,
    );
    type_error("new WeakRef.prototype.deref()");
}

#[test]
fn deref_ignores_arguments_and_public_target_properties_and_survives_integrity_changes() {
    check(
        r#"
        let target={},reference=new WeakRef(target),deref=reference.deref,hits=0;
        Object.defineProperty(reference,'target',{get(){hits++;throw 7;}});
        Object.defineProperty(reference,Symbol.toStringTag,{value:'custom'});Object.setPrototypeOf(reference,null);Object.freeze(reference);
        let hostile={get [Symbol.toPrimitive](){hits++;throw 8;}};
        deref.call(reference,hostile)===target && Reflect.apply(deref,reference,[hostile,hostile])===target &&
        deref.bind(reference)(hostile)===target && hits===0 && Object.isFrozen(reference)
        "#,
    );
}

#[test]
fn instance_surface_is_ordinary_and_the_prototype_tag_does_not_supply_the_brand() {
    // The complete descriptor graph remains covered by the existing metadata
    // snapshot; this checks its behavior with newly constructible instances.
    check(
        r#"
        let target={},reference=new WeakRef(target),prototype=WeakRef.prototype;
        if(Object.getPrototypeOf(reference)!==prototype || Reflect.ownKeys(reference).length!==0 || !Object.isExtensible(reference))throw 1;
        let tag=Object.getOwnPropertyDescriptor(prototype,Symbol.toStringTag);
        if(tag.value!=='WeakRef' || tag.writable || tag.enumerable || !tag.configurable || 'get' in tag)throw 2;
        let fake=Object.create(prototype),caught=false;
        try{prototype.deref.call(fake);}catch(e){caught=e instanceof TypeError;}
        Object.prototype.toString.call(reference)==='[object WeakRef]' &&
        Object.prototype.toString.call(fake)==='[object WeakRef]' && caught &&
        !('clear' in reference) && !('target' in reference) && !(Symbol.iterator in reference) &&
        !(Symbol.species in WeakRef) && prototype.constructor===WeakRef
        "#,
    );
}

#[test]
fn clearing_a_fresh_or_empty_realm_does_not_initialize_intrinsics() {
    let mut realm = Realm::default();
    realm.clear_kept_objects().unwrap();
    realm.clear_kept_objects().unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, 0);
    realm.eval("0").unwrap();
    realm.clear_kept_objects().unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
}

#[test]
fn construction_keeps_targets_across_evaluations_property_reads_and_collections() {
    let mut realm = Realm::default();
    let target = realm
        .eval(
            "let reference;(()=>{let target={n:7};reference=new WeakRef(target);return target;})()",
        )
        .unwrap();
    let Value::Object(handle) = &target else {
        panic!("expected object target: {target:?}")
    };
    let script = spite_parser::parse_script("({unrelated:8})").unwrap();
    for _ in 0..3 {
        realm.eval("({unrelated:7})").unwrap();
        realm.evaluate(&script).unwrap();
        realm.collect(usize::MAX).unwrap();
        assert_eq!(
            realm.read_property(&target, &JsString::from("n")),
            Ok(Value::Number(7.0))
        );
    }
    assert!(realm.collect(0).is_err());
    assert!(realm.inspect_object(handle).is_ok());
    realm.clear_kept_objects().unwrap();
    // The checkpoint itself does not collect or invalidate the object.
    assert!(realm.inspect_object(handle).is_ok());
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(handle).is_err());
    assert_eq!(realm.eval("reference.deref()"), Ok(Value::Undefined));
}

#[test]
fn live_deref_keeps_again_and_checkpoint_preserves_independent_host_roots() {
    let mut realm = Realm::default();
    let target = realm
        .eval(
            "let reference;(()=>{let target={n:7};reference=new WeakRef(target);return target;})()",
        )
        .unwrap();
    let root = realm.root_value(target.clone(), usize::MAX).unwrap();
    let clone = root.clone();
    drop(root);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(clone.value(), &target);
    assert_eq!(realm.eval("reference.deref()"), Ok(target.clone()));
    drop(clone);
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.read_property(&target, &JsString::from("n")),
        Ok(Value::Number(7.0))
    );
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("reference.deref()"), Ok(Value::Undefined));
}

#[test]
fn unreachable_weakref_records_do_not_end_the_kept_target_lifetime() {
    let mut realm = Realm::default();
    let target = realm.eval("new WeakRef({n:7}).deref()").unwrap();
    let Value::Object(handle) = &target else {
        panic!("expected object target: {target:?}")
    };
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.read_property(&target, &JsString::from("n")),
        Ok(Value::Number(7.0))
    );
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(handle).is_err());
}

#[test]
fn multiple_weakrefs_agree_after_collection_and_later_allocations_do_not_revive_them() {
    let mut realm = Realm::default();
    realm.eval("let references;(()=>{let target={};references=[new WeakRef(target),new WeakRef(target)];})()").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("references.every(reference=>reference.deref()===undefined)"),
        Ok(Value::Boolean(true))
    );
    realm
        .eval("let replacements=[];for(let i=0;i<16;i++)replacements.push({n:i});")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("references.every(reference=>reference.deref()===undefined)"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn fresh_symbol_targets_are_kept_then_expire_and_permanent_symbols_stay_valid() {
    let mut realm = Realm::default();
    realm.eval("let references;(()=>{let target=Symbol('local');references=[new WeakRef(target),new WeakRef(target)];})()").unwrap();
    for _ in 0..2 {
        realm.collect(usize::MAX).unwrap();
        assert_eq!(
            realm.eval("typeof references[0].deref()==='symbol' && references[0].deref()===references[1].deref()"),
            Ok(Value::Boolean(true))
        );
    }
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("references.every(reference=>reference.deref()===undefined)"),
        Ok(Value::Boolean(true))
    );
    realm
        .eval("let permanent=new WeakRef(Symbol.iterator)")
        .unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("permanent.deref()===Symbol.iterator"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn a_host_symbol_owner_survives_clear_and_a_later_deref_keeps_it_after_host_drop() {
    let mut realm = Realm::default();
    let target = realm.eval("let reference;(()=>{let target=Symbol('host');reference=new WeakRef(target);return target;})()").unwrap();
    let Value::Symbol(symbol) = target else {
        panic!("expected Symbol target: {target:?}")
    };
    let weak = symbol.downgrade();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("reference.deref()"),
        Ok(Value::Symbol(symbol.clone()))
    );
    drop(symbol);
    realm.collect(usize::MAX).unwrap();
    assert!(weak.upgrade().is_some());
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(weak.upgrade().is_none());
    assert_eq!(realm.eval("reference.deref()"), Ok(Value::Undefined));
}

#[test]
fn weakrefs_do_not_keep_mixed_weakmap_cycles_after_the_explicit_checkpoint() {
    let mut realm = Realm::default();
    realm.eval("let first=new WeakMap(),second=new WeakMap(),objectRef,symbolRef;(()=>{let object={},symbol=Symbol('cycle');first.set(object,symbol);second.set(symbol,{back:object});objectRef=new WeakRef(object);symbolRef=new WeakRef(symbol);})()").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("first.get(objectRef.deref())===symbolRef.deref() && second.get(symbolRef.deref()).back===objectRef.deref()"),
        Ok(Value::Boolean(true))
    );
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("objectRef.deref()===undefined && symbolRef.deref()===undefined"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn a_throwing_new_target_prototype_does_not_keep_its_valid_target() {
    let mut realm = Realm::default();
    let object = realm.eval("let target={};function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){throw 7;}});target").unwrap();
    assert_eq!(
        realm.eval("Reflect.construct(WeakRef,[target],N)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    realm.eval("target=undefined").unwrap();
    realm.collect(usize::MAX).unwrap();
    let Value::Object(handle) = object else {
        panic!("expected object target: {object:?}")
    };
    assert!(realm.inspect_object(&handle).is_err());

    let Value::Symbol(symbol) = realm.eval("target=Symbol('not-kept')").unwrap() else {
        panic!("expected Symbol target")
    };
    let weak = symbol.downgrade();
    assert_eq!(
        realm.eval("Reflect.construct(WeakRef,[target],N)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    realm.eval("target=undefined").unwrap();
    drop(symbol);
    realm.collect(usize::MAX).unwrap();
    assert!(weak.upgrade().is_none());
}

#[test]
fn abrupt_evaluation_and_host_property_failures_do_not_clear_kept_targets() {
    let mut realm = Realm::default();
    let target = realm
        .eval(
            "let reference;(()=>{let target={n:7};reference=new WeakRef(target);return target;})()",
        )
        .unwrap();
    assert!(matches!(realm.eval("throw 7"), Err(Error::Thrown(_))));
    assert!(matches!(realm.eval("let ="), Err(Error::Parse(_))));
    assert!(matches!(
        realm.eval("null.x"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert!(matches!(
        realm.read_property(&Value::Null, &JsString::from("x")),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.read_property(&target, &JsString::from("n")),
        Ok(Value::Number(7.0))
    );
    realm.clear_kept_objects().unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("reference.deref()"), Ok(Value::Undefined));
}

#[test]
fn explicit_work_abort_preserves_prior_kept_targets_and_allows_a_later_checkpoint() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5_000),
        ..Limits::default()
    });
    let target = realm.eval("let flag=0,reference;(()=>{let target={n:7};reference=new WeakRef(target);return target;})()").unwrap();
    let result = realm.eval("try{while(true){}}catch{flag=1;}finally{flag=2;}");
    assert!(matches!(result, Err(Error::Limit { .. })));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.read_property(&target, &JsString::from("n")),
        Ok(Value::Number(7.0))
    );
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("reference.deref()"), Ok(Value::Undefined));
}
