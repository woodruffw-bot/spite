use super::*;
use crate::{Limits, ObjectHandle, Value, realm_record::child_realm_fixture};

fn expose(realm: &mut Realm, name: &str, value: Value) {
    let global = realm.global_object();
    realm
        .define_builtin_property(&global, name, value, true, Span::new(0, 0))
        .unwrap();
}

fn expose_child(
    realm: &mut Realm,
    environment: &EnvironmentHandle,
    function: Value,
) -> ObjectHandle {
    let global = realm
        .objects
        .environment(environment)
        .unwrap()
        .inactive_realm
        .as_ref()
        .unwrap()
        .global_object
        .clone()
        .unwrap();
    expose(realm, "foreign", function);
    expose(realm, "child", Value::Object(global.clone()));
    global
}

#[test]
fn foreign_calls_use_creation_intrinsics_and_restore_the_caller() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        "var mark=7;function receiver(){return this;}function f(){return {array:[],boxed:Object(7),global:globalThis};}f",
    );
    expose_child(&mut realm, &environment, function);
    assert_eq!(
        realm.eval(
            r#"var mark=42;let compiled=child.Function('return mark');let result=foreign();
            let inspector=child.Object.getOwnPropertyDescriptor;
            Object.setPrototypeOf(inspector,null);
            let descriptor=Reflect.apply(inspector,null,[{x:1},'x']);
            Object.getPrototypeOf(result)===child.Object.prototype &&
            Object.getPrototypeOf(result.array)===child.Array.prototype &&
            Object.getPrototypeOf(result.boxed)===child.Number.prototype &&
            result.global===child &&
            Object.getPrototypeOf(child.receiver.call('x'))===child.String.prototype &&
            Object.getPrototypeOf(descriptor)===child.Object.prototype &&
            compiled()===7 && Object.getPrototypeOf(compiled)===child.Function.prototype &&
            Object.getPrototypeOf([])===Array.prototype"#,
        ),
        Ok(Value::Boolean(true))
    );
    assert_ne!(realm.record.global_environment.as_ref(), Some(&environment));
    assert_eq!(realm.scopes.len(), 1);
}

#[test]
fn foreign_errors_and_host_aborts_restore_the_owned_records() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(10_000),
        ..Limits::default()
    });
    realm.reserve_unsupported_global("missingHost");
    let (environment, function) = child_realm_fixture(
        &mut realm,
        "var marker={};function f(mode){if(mode===0)return null.x;\
         if(mode===1)throw marker;globalThis.started=true;\
         if(mode===2)missingHost();while(true){}}f",
    );
    expose_child(&mut realm, &environment, function);
    let parent = realm.record.global_environment.clone().unwrap();
    for source in [
        "try{foreign(0)}catch(e){e instanceof child.TypeError && !(e instanceof TypeError)}",
        "try{child.Object.keys(null)}catch(e){e instanceof child.TypeError && !(e instanceof TypeError)}",
        "try{child.Function('return (')}catch(e){e instanceof child.SyntaxError && !(e instanceof SyntaxError)}",
        "try{foreign(1)}catch(e){e===child.marker}",
    ] {
        assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
    }
    assert!(matches!(
        realm.eval("var effects=0;try{foreign(2)}catch(e){effects=1}finally{effects=2}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(
        realm.eval("effects===0 && child.started===true"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval("child.started=false;try{foreign(3)}catch(e){effects=1}finally{effects=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("effects===0 && child.started===true"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.record.global_environment.as_ref(), Some(&parent));
    let Value::Object(local) = realm.eval("(function(){})").unwrap() else {
        panic!()
    };
    assert_eq!(
        realm.objects.inspect(&local).unwrap().function_realm(),
        Some(&parent)
    );
    assert_eq!(realm.call_depth, 0);
    assert_eq!(realm.scopes.len(), 1);
}

#[test]
fn cross_realm_tail_calls_restore_each_record_before_forwarding() {
    let mut realm = Realm::default();
    realm
        .eval("const expected=globalThis;function parent(n){'use strict';return n===0?globalThis===expected:child.foreign(n-1);}")
        .unwrap();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        "const expected=globalThis;function foreign(n){'use strict';return n===0?globalThis===expected:parent.parent(n-1);}foreign",
    );
    let global = expose_child(&mut realm, &environment, function);
    let parent_global = realm.global_object();
    realm
        .define_builtin_property(
            &global,
            "parent",
            Value::Object(parent_global),
            true,
            Span::new(0, 0),
        )
        .unwrap();
    assert_eq!(
        realm.eval("parent(4096) && child.foreign(4097)"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.call_depth, 0);
    assert_eq!(realm.evaluation_depth, 0);
    assert_eq!(realm.scopes.len(), 1);
    assert!(
        realm
            .objects
            .environment(realm.record.global_environment.as_ref().unwrap())
            .unwrap()
            .inactive_realm
            .is_none()
    );
    assert!(
        realm
            .objects
            .environment(&environment)
            .unwrap()
            .inactive_realm
            .is_some()
    );
}

#[test]
fn proxy_bound_calls_and_template_caches_preserve_their_realms() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        "function tag(t){return t;}function f(){return tag`site`;}f",
    );
    expose_child(&mut realm, &environment, function);
    assert_eq!(
        realm.eval(
            r#"function tag(t){return t;}let own=tag`site`;
            let first=foreign(),second=foreign();
            let wrapped=new Proxy(foreign,{apply(target,receiver,args){
                return Object.getPrototypeOf(args)===Array.prototype &&
                    Object.getPrototypeOf(target())===child.Array.prototype;
            }});
            let bound=foreign.bind(null);Object.setPrototypeOf(bound,null);
            wrapped(1,2) && bound()===first && first===second && first!==own &&
            Object.getPrototypeOf(first)===child.Array.prototype &&
            Object.getPrototypeOf(own)===Array.prototype"#,
        ),
        Ok(Value::Boolean(true))
    );
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("foreign()===first"), Ok(Value::Boolean(true)));
}

#[test]
fn constructor_fallbacks_use_new_target_intrinsics_without_changing_creation_realm() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        "var asyncCtor=(async function(){}).constructor,genCtor=(function*(){}).constructor,\
         asyncGenCtor=(async function*(){}).constructor;function Foreign(){}Foreign",
    );
    expose_child(&mut realm, &environment, function);
    assert_eq!(
        realm.eval(
            r#"foreign.prototype=0;
            let rows=[
                [Object,[],child.Object.prototype],
                [Function,['return 1'],child.Function.prototype],
                [(async function(){}).constructor,['return 1'],child.asyncCtor.prototype],
                [(function*(){}).constructor,['return 1'],child.genCtor.prototype],
                [(async function*(){}).constructor,['return 1'],child.asyncGenCtor.prototype],
                [Boolean,[1],child.Boolean.prototype],
                [Number,[7],child.Number.prototype],
                [String,['x'],child.String.prototype],
                [Date,[0],child.Date.prototype],
                [RegExp,['x'],child.RegExp.prototype],
                [Array,[2],child.Array.prototype],
                [ArrayBuffer,[4],child.ArrayBuffer.prototype],
                [SharedArrayBuffer,[4],child.SharedArrayBuffer.prototype],
                [DataView,[new ArrayBuffer(4)],child.DataView.prototype],
                [Int8Array,[2],child.Int8Array.prototype],
                [Uint8Array,[2],child.Uint8Array.prototype],
                [Uint8ClampedArray,[2],child.Uint8ClampedArray.prototype],
                [Int16Array,[2],child.Int16Array.prototype],
                [Uint16Array,[2],child.Uint16Array.prototype],
                [Int32Array,[2],child.Int32Array.prototype],
                [Uint32Array,[2],child.Uint32Array.prototype],
                [BigInt64Array,[2],child.BigInt64Array.prototype],
                [BigUint64Array,[2],child.BigUint64Array.prototype],
                [Float16Array,[2],child.Float16Array.prototype],
                [Float32Array,[2],child.Float32Array.prototype],
                [Float64Array,[2],child.Float64Array.prototype],
                [Map,[],child.Map.prototype],[Set,[],child.Set.prototype],
                [WeakMap,[],child.WeakMap.prototype],[WeakSet,[],child.WeakSet.prototype],
                [WeakRef,[{}],child.WeakRef.prototype],
                [FinalizationRegistry,[()=>{}],child.FinalizationRegistry.prototype],
                [Error,['x'],child.Error.prototype],[EvalError,['x'],child.EvalError.prototype],
                [RangeError,['x'],child.RangeError.prototype],
                [ReferenceError,['x'],child.ReferenceError.prototype],
                [SyntaxError,['x'],child.SyntaxError.prototype],
                [TypeError,['x'],child.TypeError.prototype],[URIError,['x'],child.URIError.prototype],
                [AggregateError,[[]],child.AggregateError.prototype],
                [SuppressedError,[1,2,'x'],child.SuppressedError.prototype],
                [Promise,[()=>{}],child.Promise.prototype],
                [Iterator,[],child.Iterator.prototype],
                [DisposableStack,[],child.DisposableStack.prototype],
                [AsyncDisposableStack,[],child.AsyncDisposableStack.prototype]
            ];let failures=[];
            for(let [ctor,args,expected] of rows){
                let result=Reflect.construct(ctor,args,foreign);
                if(Object.getPrototypeOf(result)!==expected)failures.push(ctor.name);
            }
            function C(){this.createdIn=globalThis;}
            let ordinary=Reflect.construct(C,[],foreign);
            if(Object.getPrototypeOf(ordinary)!==child.Object.prototype||ordinary.createdIn!==globalThis)failures.push('ordinary');
            let compiled=Reflect.construct(Function,['return globalThis'],foreign);
            if(compiled()!==globalThis)failures.push('function-creation-realm');
            let view=Reflect.construct(Uint8Array,[2],foreign);
            if(Object.getPrototypeOf(view.buffer)!==ArrayBuffer.prototype)failures.push('buffer-allocation-realm');
            let aggregate=Reflect.construct(AggregateError,[[]],foreign);
            if(Object.getPrototypeOf(aggregate.errors)!==Array.prototype)failures.push('errors-allocation-realm');
            let resolving=false;
            Reflect.construct(Promise,[resolve=>{resolving=Object.getPrototypeOf(resolve)===Function.prototype;resolve(7);}],foreign);
            if(!resolving)failures.push('promise-executor-realm');
            let generator=Reflect.construct((function*(){}).constructor,['yield 1'],foreign);generator.prototype=0;
            if(Object.getPrototypeOf(generator())!==Object.getPrototypeOf((function*(){}).prototype))failures.push('generator-creation-realm');
            let asyncGenerator=Reflect.construct((async function*(){}).constructor,['yield 1'],foreign);asyncGenerator.prototype=0;
            if(Object.getPrototypeOf(asyncGenerator())!==Object.getPrototypeOf((async function*(){}).prototype))failures.push('async-generator-creation-realm');
            JSON.stringify({checked:rows.length,failures})"#,
        ),
        Ok(Value::String("{\"checked\":45,\"failures\":[]}".into()))
    );
}

#[test]
fn fallback_realm_queries_follow_targets_after_the_single_prototype_get() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(&mut realm, "function Foreign(){}Foreign");
    expose_child(&mut realm, &environment, function);
    assert_eq!(
        realm.eval(
            r#"foreign.prototype=0;Object.setPrototypeOf(foreign,null);
            let bound=Function.prototype.bind.call(foreign,null);
            let symbol=Function.prototype.bind.call(child.Symbol,null);
            let bigint=Function.prototype.bind.call(child.BigInt,null);
            let reads=0,proxy=new Proxy(foreign,{get(t,k){if(k==='prototype'){reads++;return 0;}throw 7;}});
            let a=Reflect.construct(Number,[8],bound),b=Reflect.construct(Object,[],proxy);
            let symbols=Reflect.construct(Number,[9],symbol),bigints=Reflect.construct(Boolean,[1],bigint);
            let before=reads,coercion=false;
            try{Reflect.construct(Number,[Symbol()],proxy)}catch(e){coercion=e instanceof TypeError && reads===before;}
            let effects=0,pair=Proxy.revocable(foreign,{get(t,k){if(k==='prototype'){pair.revoke();return 0;}throw 7;}});
            function C(){effects++;}let revoked=false;
            try{Reflect.construct(C,[],pair.proxy)}catch(e){revoked=e instanceof TypeError && effects===0;}
            let chosen={},other=Proxy.revocable(foreign,{get(t,k){if(k==='prototype'){other.revoke();return chosen;}throw 7;}});
            let explicit=Reflect.construct(C,[],other.proxy);
            Object.getPrototypeOf(a)===child.Number.prototype &&
            Object.getPrototypeOf(b)===child.Object.prototype && reads===1 &&
            Object.getPrototypeOf(symbols)===child.Number.prototype &&
            Object.getPrototypeOf(bigints)===child.Boolean.prototype &&
            coercion && revoked && effects===1 && Object.getPrototypeOf(explicit)===chosen"#,
        ),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn array_species_normalizes_only_the_other_realms_exact_intrinsic_array() {
    let mut realm = Realm::default();
    let (environment, function) =
        child_realm_fixture(&mut realm, "var array=[1,2];function Foreign(){}Foreign");
    expose_child(&mut realm, &environment, function);
    assert_eq!(
        realm.eval(
            r#"let original=child.Array,reads=0;
            Object.defineProperty(original,Symbol.species,{get(){reads++;throw 42;}});
            child.Array=function replacement(){};
            let mapped=Array.prototype.map.call(child.array,x=>x+1);
            let normalized=Object.getPrototypeOf(mapped)===Array.prototype && mapped.join(',')==='2,3' && reads===0;
            let backwards=original.prototype.map.call([3,4],x=>x*2);
            let reverse=Object.getPrototypeOf(backwards)===original.prototype && backwards.join(',')==='6,8' && reads===0;
            child.array.constructor=new Proxy(original,{});let proxy=false;
            try{Array.prototype.map.call(child.array,x=>x)}catch(e){proxy=e===42 && reads===1;}
            let bound=Function.prototype.bind.call(original,null),used=0;
            function C(n){used++;this.length=n;}
            Object.defineProperty(bound,Symbol.species,{get(){reads++;return C;}});
            child.array.constructor=bound;
            let custom=Array.prototype.map.call(child.array,x=>x);
            let boundUsed=Object.getPrototypeOf(custom)===C.prototype && reads===2 && used===1;
            let pair=Proxy.revocable(original,{get(){throw 99;}});pair.revoke();child.array.constructor=pair.proxy;
            let revoked=false;try{Array.prototype.map.call(child.array,x=>x)}catch(e){revoked=e instanceof TypeError && reads===2;}
            normalized && reverse && proxy && boundUsed && revoked"#,
        ),
        Ok(Value::Boolean(true))
    );
}
