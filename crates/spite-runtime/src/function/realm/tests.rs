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

#[test]
fn iterative_realm_transitions_unwind_foreign_errors_to_the_original_caller() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(&mut realm, "function Foreign(){}Foreign");
    expose_child(&mut realm, &environment, function);
    let parent = realm.record.global_environment.clone().unwrap();
    let span = Span::new(0, 0);
    let mut transitions = Vec::new();
    for index in 0..2049 {
        let target = if index % 2 == 0 {
            &environment
        } else {
            &parent
        };
        transitions.push(realm.enter_function_realm(target, span).unwrap());
    }
    let mut result = realm.eval("null.x");
    for transition in transitions.into_iter().rev() {
        result = realm.leave_function_realm(transition, result);
    }
    let Err(Error::Thrown(error)) = result else {
        panic!("the foreign exception must be materialized before its Realm is removed")
    };
    expose(&mut realm, "caught", error);
    assert_eq!(
        realm.eval("caught instanceof child.TypeError && !(caught instanceof TypeError)"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.record.global_environment.as_ref(), Some(&parent));
    assert_eq!(realm.scopes.len(), 1);
}

#[test]
fn builtin_construction_uses_its_creation_realm_for_native_errors() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(&mut realm, "function Foreign(){}Foreign");
    expose_child(&mut realm, &environment, function);
    assert_eq!(
        realm.eval(
            r#"let failures=[];
            for(let make of [()=>new child.Symbol(),()=>new child.BigInt(),
                ()=>new child.Number(Symbol()),()=>new child.String(Symbol()),
                ()=>new child.WeakMap(1),()=>new child.Promise(1),()=>new child.Iterator()]){
                try{make();failures.push('missing-type-error')}catch(e){
                    if(!(e instanceof child.TypeError)||e instanceof TypeError)failures.push('type-error-realm');
                }
            }
            for(let make of [()=>new child.Array(-1),()=>new child.ArrayBuffer(-1),
                ()=>new child.SharedArrayBuffer(-1),()=>new child.Int8Array(-1),
                ()=>new child.DataView(new ArrayBuffer(1),2)]){
                try{make();failures.push('missing-range-error')}catch(e){
                    if(!(e instanceof child.RangeError)||e instanceof RangeError)failures.push('range-error-realm');
                }
            }
            let callback=false;
            try{new child.Number({[Symbol.toPrimitive](){return null.x;}})}catch(e){
                callback=e instanceof TypeError && !(e instanceof child.TypeError);
            }
            if(!callback)failures.push('callback-error-realm');
            let reads=0,coercions=0,proxy=new Proxy(foreign,{get(){reads++;throw 7;}});
            try{Reflect.construct(child.Symbol,[{[Symbol.toPrimitive](){coercions++;return 1;}}],proxy)}catch(e){
                if(!(e instanceof child.TypeError)||reads!==0||coercions!==0)failures.push('symbol-order');
            }
            JSON.stringify(failures)"#,
        ),
        Ok(Value::String("[]".into()))
    );
}

#[test]
fn builtin_creation_realm_and_new_target_realm_remain_independent() {
    let mut realm = Realm::default();
    let (environment, function) =
        child_realm_fixture(&mut realm, "var mark=7;function Foreign(){}Foreign");
    expose_child(&mut realm, &environment, function);
    let (third_environment, third_target) =
        child_realm_fixture(&mut realm, "function Third(){}Third");
    let third = realm
        .objects
        .environment(&third_environment)
        .unwrap()
        .inactive_realm
        .as_ref()
        .unwrap()
        .global_object
        .clone()
        .unwrap();
    expose(&mut realm, "third", Value::Object(third));
    expose(&mut realm, "thirdTarget", third_target);
    assert_eq!(
        realm.eval(
            r#"var mark=42;thirdTarget.prototype=0;
            let compiled=new child.Function('return mark');
            let mixed=Reflect.construct(child.Function,['return globalThis'],thirdTarget);
            let view=Reflect.construct(child.Uint8Array,[2],thirdTarget);
            let aggregate=Reflect.construct(child.AggregateError,[[]],thirdTarget);
            let resolving=false,promise=Reflect.construct(child.Promise,[resolve=>{
                resolving=Object.getPrototypeOf(resolve)===child.Function.prototype;resolve(7);
            }],thirdTarget);
            let pair=Proxy.revocable(thirdTarget,{get(t,k){if(k==='prototype'){pair.revoke();return 0;}throw 7;}});
            let builtinError=false;try{Reflect.construct(child.Object,[],pair.proxy)}catch(e){
                builtinError=e instanceof child.TypeError && !(e instanceof TypeError);
            }
            let source=Proxy.revocable(thirdTarget,{get(t,k){if(k==='prototype'){source.revoke();return 0;}throw 7;}});
            let sourceError=false;try{Reflect.construct(foreign,[],source.proxy)}catch(e){
                sourceError=e instanceof TypeError && !(e instanceof child.TypeError);
            }
            compiled()===7 && mixed()===child &&
            Object.getPrototypeOf(mixed)===third.Function.prototype &&
            Object.getPrototypeOf(view)===third.Uint8Array.prototype &&
            Object.getPrototypeOf(view.buffer)===child.ArrayBuffer.prototype &&
            Object.getPrototypeOf(aggregate)===third.AggregateError.prototype &&
            Object.getPrototypeOf(aggregate.errors)===child.Array.prototype &&
            Object.getPrototypeOf(promise)===third.Promise.prototype && resolving && builtinError && sourceError &&
            Object.getPrototypeOf([])===Array.prototype"#,
        ),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn recursive_foreign_builtin_construction_keeps_the_two_mebibyte_guard() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut realm = Realm::default();
            let (environment, function) = child_realm_fixture(&mut realm, "function Foreign(){}Foreign");
            expose_child(&mut realm, &environment, function);
            let parent = realm.record.global_environment.clone().unwrap();
            assert!(matches!(
                realm.eval("let effects=0,calls=0,value={[Symbol.toPrimitive](){calls++;return new child.Number(value).valueOf();}};try{new child.Number(value)}catch(e){effects=1}finally{effects=2}"),
                Err(Error::Limit { .. })
            ));
            assert_eq!(realm.eval("effects===0 && calls>0"), Ok(Value::Boolean(true)));
            assert_eq!(realm.record.global_environment.as_ref(), Some(&parent));
            assert_eq!(realm.call_depth, 0);
            assert_eq!(realm.scopes.len(), 1);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn source_constructor_bodies_and_fields_use_their_creation_realm() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        r#"var Base=class {field=[];constructor(arg=[]){this.arg=arg;this.body={};}};
        var Ordinary=function(arg=[]){this.arg=arg;this.body={};};
        var Default=class {field=[];};
        var Derived=class extends Base {derived=[];constructor(arg=[]){super();this.derivedArg=arg;}};
        var derive=function(B){return class extends B {childField=[];constructor(){super();this.childBody={};}};};
        Base"#,
    );
    expose_child(&mut realm, &environment, function);
    assert_eq!(
        realm.eval(
            r#"let failures=[];
            for(let C of [child.Base,child.Ordinary,child.Default,child.Derived]){
                let x=new C;
                if(Object.getPrototypeOf(x)!==C.prototype)failures.push('instance');
                for(let key of ['field','arg','derived','derivedArg']){
                    if(key in x && Object.getPrototypeOf(x[key])!==child.Array.prototype)failures.push(key);
                }
                if('body' in x && Object.getPrototypeOf(x.body)!==child.Object.prototype)failures.push('body');
            }
            class Parent extends child.Base {parentField=[];constructor(){super();this.parentBody={};}}
            let p=new Parent;
            if(Object.getPrototypeOf(p.field)!==child.Array.prototype ||
                Object.getPrototypeOf(p.parentField)!==Array.prototype ||
                Object.getPrototypeOf(p.parentBody)!==Object.prototype)failures.push('parent-derived');
            class Root {rootField=[];constructor(){this.rootBody={};}}
            let C=child.derive(Root),c=new C;
            if(Object.getPrototypeOf(c.rootField)!==Array.prototype ||
                Object.getPrototypeOf(c.rootBody)!==Object.prototype ||
                Object.getPrototypeOf(c.childField)!==child.Array.prototype ||
                Object.getPrototypeOf(c.childBody)!==child.Object.prototype)failures.push('child-derived');
            Object.getPrototypeOf([])===Array.prototype && JSON.stringify(failures)"#
        ),
        Ok(Value::String("[]".into()))
    );
    assert_eq!(realm.scopes.len(), 1);
    assert_eq!(realm.call_depth, 0);
}

#[test]
fn source_and_default_constructor_allocation_have_distinct_caller_realms() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        r#"var Ordinary=function(){};
        var Explicit=class {constructor(){}};
        var Default=class {};
        var DefaultDerived=class extends Ordinary {};
        var makeDefault=function(B){return class extends B {};};
        Ordinary"#,
    );
    expose_child(&mut realm, &environment, function);
    assert_eq!(
        realm.eval(
            r#"let failures=[];
            function allocationFailure(C,E,label){
                let pair=Proxy.revocable(function(){},{get(t,k){if(k==='prototype'){pair.revoke();return 0;}return t[k];}});
                try{Reflect.construct(C,[],pair.proxy);failures.push(label+':missing');}
                catch(e){if(!(e instanceof E))failures.push(label+':realm');}
            }
            allocationFailure(child.Ordinary,TypeError,'ordinary');
            allocationFailure(child.Explicit,TypeError,'explicit');
            allocationFailure(child.Default,child.TypeError,'default');
            allocationFailure(child.DefaultDerived,child.TypeError,'default-source-super');
            class ParentExplicit {constructor(){}}
            allocationFailure(child.makeDefault(ParentExplicit),child.TypeError,'foreign-source-super');
            let D=child.makeDefault(ParentExplicit);
            Object.setPrototypeOf(D,null);
            try{new D;failures.push('invalid-super:missing');}
            catch(e){if(!(e instanceof child.TypeError))failures.push('invalid-super:realm');}
            let pair=Proxy.revocable(ParentExplicit,{}),P=child.makeDefault(pair.proxy);
            pair.revoke();
            try{new P;failures.push('revoked-super:missing');}
            catch(e){if(!(e instanceof child.TypeError))failures.push('revoked-super:realm');}
            JSON.stringify(failures)"#
        ),
        Ok(Value::String("[]".into()))
    );
    assert_eq!(realm.scopes.len(), 1);
    assert_eq!(realm.call_depth, 0);
}

#[test]
fn source_constructor_late_errors_and_tail_forwarding_restore_the_caller_realm() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        r#"var Primitive=class extends Object {constructor(){return 1;}};
        var Missing=class extends Object {constructor(){}};
        var Body=class extends Object {constructor(){null.x;}};
        var Parameter=class extends Object {constructor(x=this){super();}};
        var Field=class {x=null.x;constructor(){}};
        var Callback=class {constructor(x=callback()){}};
        var tail,callback;
        var Tail=class extends Object {constructor(){return tail();}};
        Primitive"#,
    );
    expose_child(&mut realm, &environment, function);
    assert_eq!(
        realm.eval(
            r#"let failures=[];
            function expect(C,E,label){try{new C;failures.push(label+':missing');}catch(e){if(!(e instanceof E))failures.push(label+':realm');}}
            child.callback=function(){return null.x;};
            let pair=Proxy.revocable(function(){return {};},{});child.tail=pair.proxy;pair.revoke();
            expect(child.Primitive,TypeError,'primitive');
            expect(child.Missing,ReferenceError,'missing');
            expect(child.Body,child.TypeError,'body');
            expect(child.Parameter,child.ReferenceError,'parameter');
            expect(child.Field,child.TypeError,'field');
            expect(child.Callback,TypeError,'callback');
            expect(child.Tail,TypeError,'tail');
            class ParentPrimitive extends Object {constructor(){return 1;}}
            class ParentMissing extends Object {constructor(){}}
            class ParentBody extends Object {constructor(){null.x;}}
            for(let [C,E,label] of [[ParentPrimitive,child.TypeError,'parent-primitive'],
                [ParentMissing,child.ReferenceError,'parent-missing'],[ParentBody,TypeError,'parent-body']]){
                try{child.Reflect.construct(C,[]);failures.push(label+':missing');}
                catch(e){if(!(e instanceof E))failures.push(label+':realm');}
            }
            let Outer=child.Function('B','return class extends B {}')(child.Primitive);
            expect(Outer,child.TypeError,'suspended-default');
            JSON.stringify(failures)"#
        ),
        Ok(Value::String("[]".into()))
    );
    assert_eq!(
        realm.record.global_environment.as_ref(),
        realm.scopes.first()
    );
    assert_eq!(realm.call_depth, 0);
}

#[test]
fn deep_default_constructor_realms_and_source_recursion_restore_on_all_exits() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut realm = Realm::default();
            let (environment, function) = child_realm_fixture(
                &mut realm,
                r#"var record,rootCtor;
                var make=function(B){return class extends B {x=[];check=record(this.x,'child');};};
                var Recursive=function(){new rootCtor;};
                make"#,
            );
            expose_child(&mut realm, &environment, function);
            assert_eq!(realm.eval(
                r#"let count=0,wrong=0,effects=0;
                function record(a,kind){count++;if(Object.getPrototypeOf(a)!==(kind==='child'?child.Array.prototype:Array.prototype))wrong++;}
                child.record=record;
                function make(B){return class extends B {x=[];check=record(this.x,'parent');};}
                let C=class {};
                for(let i=0;i<2049;i++)C=(i%2?make:child.make)(C);
                let x=new C;
                count===2049 && wrong===0 && Object.getPrototypeOf(x.x)===child.Array.prototype"#
            ),Ok(Value::Boolean(true)));
            assert_eq!(realm.eval(
                r#"function Frozen(){return Object.freeze({});}
                let D=Frozen;for(let i=0;i<2049;i++)D=(i%2?make:child.make)(D);
                let caught=false;try{new D;}catch(e){caught=e instanceof child.TypeError;}
                caught && Object.getPrototypeOf([])===Array.prototype"#
            ),Ok(Value::Boolean(true)));
            realm.limits.max_steps=Some(1000);
            assert!(matches!(realm.eval("try{new C;}catch(e){effects++;}finally{effects++;}"),Err(Error::Limit{..})));
            assert_eq!(realm.record.global_environment.as_ref(), realm.scopes.first());
            assert_eq!(realm.scopes.len(),1);
            assert_eq!(realm.call_depth,0);
            realm.limits.max_steps=None;
            assert!(matches!(realm.eval(
                "function Root(){new child.Recursive;}child.rootCtor=Root;try{new Root;}catch(e){effects++;}finally{effects++;}"
            ),Err(Error::Limit{..})));
            assert_eq!(realm.eval("effects"),Ok(Value::Number(0.0)));
            assert_eq!(realm.record.global_environment.as_ref(), realm.scopes.first());
            assert_eq!(realm.scopes.len(),1);
            assert_eq!(realm.call_depth,0);
        }).unwrap().join().unwrap();
}

#[test]
fn borrowed_generator_methods_resume_the_defining_realm_and_keep_result_boundaries() {
    std::thread::Builder::new().stack_size(2*1024*1024).spawn(|| {
        let mut realm=Realm::default();
        let (environment,function)=child_realm_fixture(&mut realm,r#"
            var GP=Object.getPrototypeOf(function*(){}.prototype);
            var gen=(function*(){yield [];return [];})();
            var bad=(function*(){null.x;})();
            var recursive,again;
            function noop(){}noop
        "#);
        expose_child(&mut realm,&environment,function);
        assert_eq!(realm.eval(r#"
            let failures=[],GP=Object.getPrototypeOf(function*(){}.prototype);
            let local=(function*(){yield [];return [];})();
            let first=child.GP.next.call(local),last=child.GP.next.call(local),done=child.GP.next.call(local);
            if(Object.getPrototypeOf(first)!==Object.prototype || Object.getPrototypeOf(first.value)!==Array.prototype)failures.push('parent-yield');
            if(Object.getPrototypeOf(last)!==child.Object.prototype || Object.getPrototypeOf(last.value)!==Array.prototype || !last.done)failures.push('parent-return');
            if(Object.getPrototypeOf(done)!==child.Object.prototype || !done.done)failures.push('completed');
            first=GP.next.call(child.gen);last=GP.next.call(child.gen);
            if(Object.getPrototypeOf(first)!==child.Object.prototype || Object.getPrototypeOf(first.value)!==child.Array.prototype)failures.push('child-yield');
            if(Object.getPrototypeOf(last)!==Object.prototype || Object.getPrototypeOf(last.value)!==child.Array.prototype)failures.push('child-return');
            try{GP.next.call(child.bad);failures.push('child-error:missing');}catch(e){if(!(e instanceof child.TypeError))failures.push('child-error:realm');}
            try{child.GP.next.call((function*(){null.x;})());failures.push('parent-error:missing');}catch(e){if(!(e instanceof TypeError))failures.push('parent-error:realm');}
            let abrupt=(function*(){try{yield 1;}finally{yield [];}})();
            child.GP.next.call(abrupt);let r=child.GP.return.call(abrupt,7),q=child.GP.next.call(abrupt);
            if(r.done || Object.getPrototypeOf(r)!==Object.prototype || Object.getPrototypeOf(r.value)!==Array.prototype || q.value!==7 || !q.done || Object.getPrototypeOf(q)!==child.Object.prototype)failures.push('return-finally');
            let marker={},fresh=(function*(){throw 1;})();
            let skipped=child.GP.return.call(fresh,marker);
            if(skipped.value!==marker || !skipped.done || Object.getPrototypeOf(skipped)!==child.Object.prototype)failures.push('return-start');
            JSON.stringify(failures)
        "#),Ok(Value::String("[]".into())));
        assert!(matches!(realm.eval(r#"
            let effects=0;
            function* reenter(){child.GP.next.call(reenter());yield 1;}
            try{child.GP.next.call(reenter());}catch(e){effects++;}finally{effects++;}
        "#),Err(Error::Limit{..})));
        assert_eq!(realm.eval("effects"),Ok(Value::Number(0.0)));
        assert_eq!(realm.scopes.len(),1);
        assert_eq!(realm.call_depth,0);
        assert_eq!(realm.record.global_environment.as_ref(),realm.scopes.first());
    }).unwrap().join().unwrap();
}

#[test]
fn borrowed_async_generator_methods_keep_source_allocations_and_errors_in_their_realm() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        r#"
        var GP=Object.getPrototypeOf(async function*(){}.prototype);
        var gen=(async function*(){yield [];return [];})();
        var bad=(async function*(){null.x;})();
        function noop(){}noop
    "#,
    );
    expose_child(&mut realm, &environment, function);
    assert_eq!(realm.eval(r#"
        let GP=Object.getPrototypeOf(async function*(){}.prototype),results=[],errors=[];
        let local=(async function*(){yield [];return [];})();
        let p=child.GP.next.call(local),q=GP.next.call(child.gen);
        p.then(r=>results.push(['parent-yield',r]));q.then(r=>results.push(['child-yield',r]));
        child.GP.next.call((async function*(){null.x;})()).catch(e=>errors.push(e instanceof TypeError));
        GP.next.call(child.bad).catch(e=>errors.push(e instanceof child.TypeError));
        let empty=child.GP.next.call((async function*(){})());empty.then(r=>results.push(['empty',r]));
        Object.getPrototypeOf(p)===child.Promise.prototype && Object.getPrototypeOf(q)===Promise.prototype
    "#),Ok(Value::Boolean(true)));
    drain_realm_test_jobs(&mut realm);
    assert_eq!(realm.eval(r#"
        errors.length===2 && errors.every(Boolean) && results.length===3 &&
        results.every(([kind,r])=>kind==='empty' ? r.done && Object.getPrototypeOf(r)===child.Object.prototype :
            !r.done && Object.getPrototypeOf(r.value)===(kind==='parent-yield'?Array.prototype:child.Array.prototype))
    "#),Ok(Value::Boolean(true)));
    realm.eval("child.GP.next.call(local).then(r=>results.push(['parent-return',r]));GP.next.call(child.gen).then(r=>results.push(['child-return',r]));").unwrap();
    drain_realm_test_jobs(&mut realm);
    assert_eq!(realm.eval("results.length===5 && results.filter(([k])=>k.endsWith('return')).every(([k,r])=>r.done && Object.getPrototypeOf(r.value)===(k==='parent-return'?Array.prototype:child.Array.prototype))"),Ok(Value::Boolean(true)));
    assert_eq!(
        realm.record.global_environment.as_ref(),
        realm.scopes.first()
    );
    assert_eq!(realm.scopes.len(), 1);
    assert_eq!(realm.call_depth, 0);
}

fn drain_realm_test_jobs(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            crate::PromiseJobOutcome::NoJob => break,
            crate::PromiseJobOutcome::Completed => (),
            outcome => panic!("unexpected job result: {outcome:?}"),
        }
    }
}

#[test]
fn linked_foreign_module_context_uses_its_defining_global_environment() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(&mut realm, "function noop(){}noop");
    expose_child(&mut realm, &environment, function);
    let module=realm.with_function_realm(&environment,Span::new(0,0),|realm|{
        let module=realm.create_module("globalThis.before=[];await 0;globalThis.after={};export function value(){return [];}" )?;
        realm.link_module(&module)?;
        Ok(module)
    }).unwrap();
    realm.evaluate_module_record(&module).unwrap();
    assert_eq!(realm.eval("Object.getPrototypeOf(child.before)===child.Array.prototype && typeof before==='undefined'"),Ok(Value::Boolean(true)));
    drain_realm_test_jobs(&mut realm);
    let namespace = realm.module_namespace(&module).unwrap();
    expose(&mut realm, "foreignNamespace", namespace);
    assert_eq!(realm.eval("Object.getPrototypeOf(child.after)===child.Object.prototype && Object.getPrototypeOf(foreignNamespace.value())===child.Array.prototype && typeof after==='undefined' && Object.getPrototypeOf([])===Array.prototype"),Ok(Value::Boolean(true)));
    assert_eq!(
        realm.record.global_environment.as_ref(),
        realm.scopes.first()
    );
    assert_eq!(realm.scopes.len(), 1);
    assert_eq!(realm.call_depth, 0);
}

#[test]
fn thenable_jobs_create_resolving_functions_and_proxy_arguments_in_the_captured_realm() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        r#"
        var seen=[];
        function then(resolve,reject){seen.push(Object.getPrototypeOf(resolve)===Function.prototype && Object.getPrototypeOf(reject)===Function.prototype);resolve(7);}
        Object.setPrototypeOf(then,null);then
    "#,
    );
    expose_child(&mut realm, &environment, function);
    assert_eq!(realm.eval(r#"
        let values=[],trapRealm=false;
        let bound=Function.prototype.bind.call(foreign,null);
        let wrapped=new Proxy(bound,{apply(t,v,args){trapRealm=Object.getPrototypeOf(args)===child.Array.prototype;return Reflect.apply(t,v,args);}});
        Promise.resolve({then:foreign}).then(x=>values.push(x));
        Promise.resolve({then:wrapped}).then(x=>values.push(x));
        child.seen.length===0 && values.length===0
    "#),Ok(Value::Boolean(true)));
    drain_realm_test_jobs(&mut realm);
    assert_eq!(realm.eval("child.seen.length===2 && child.seen.every(Boolean) && values.join(',')==='7,7' && trapRealm && Object.getPrototypeOf([])===Array.prototype"),Ok(Value::Boolean(true)));
    assert_eq!(
        realm.record.global_environment.as_ref(),
        realm.scopes.first()
    );
}

#[test]
fn promise_job_realms_capture_revocation_at_enqueue_and_pending_settlement() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        r#"
        var release,pending=new Promise(r=>release=r);
        function handler(x){return x;}handler
    "#,
    );
    expose_child(&mut realm, &environment, function);
    realm
        .eval(
            r#"
        let seen=[];
        function watch(p,E,label){p.catch(e=>seen.push([label,e instanceof E]));}
        let later=Proxy.revocable(foreign,{});
        watch(Promise.resolve(1).then(later.proxy),child.TypeError,'reaction-later');
        watch(Promise.resolve({then:later.proxy}),child.TypeError,'thenable-later');
        later.revoke();
        let earlier=Proxy.revocable(foreign,{});earlier.revoke();
        watch(Promise.resolve(1).then(earlier.proxy),TypeError,'reaction-before-parent');
        watch(child.Promise.resolve(1).then(earlier.proxy),child.TypeError,'reaction-before-child');
        watch(Promise.resolve({then:earlier.proxy}),TypeError,'thenable-before-parent');
        watch(child.Promise.resolve({then:earlier.proxy}),child.TypeError,'thenable-before-child');
        let pending=Proxy.revocable(x=>x,{});
        watch(child.pending.then(pending.proxy),child.TypeError,'pending-settlement');
        pending.revoke();child.release(7);
    "#,
        )
        .unwrap();
    drain_realm_test_jobs(&mut realm);
    assert_eq!(
        realm.eval(
            "seen.length===7 && seen.every(x=>x[1]) && Object.getPrototypeOf([])===Array.prototype"
        ),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        realm.record.global_environment.as_ref(),
        realm.scopes.first()
    );
    assert_eq!(realm.call_depth, 0);
    assert_eq!(realm.scopes.len(), 1);
}

#[test]
fn queued_job_realm_retains_an_inactive_record_after_proxy_revocation_and_gc() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        "function then(resolve){resolve(7);}Object.setPrototypeOf(then,null);then",
    );
    let expected = realm
        .objects
        .environment(&environment)
        .unwrap()
        .inactive_realm
        .as_ref()
        .unwrap()
        .intrinsics
        .as_ref()
        .unwrap()
        .errors
        .entries
        .iter()
        .find(|error| error.kind == crate::function::error::ErrorConstructor::TypeError)
        .unwrap()
        .prototype
        .clone();
    expose(&mut realm, "foreign", function);
    realm.eval("var pair=Proxy.revocable(foreign,{}),p=Promise.resolve({then:pair.proxy}),reason;p.catch(e=>reason=e);delete globalThis.foreign;pair.revoke();").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(realm.objects.environment(&environment).is_ok());
    assert!(realm.objects.inspect(&expected).is_ok());
    drain_realm_test_jobs(&mut realm);
    let Value::Object(reason) = realm.eval("reason").unwrap() else {
        panic!("missing rejection")
    };
    assert_eq!(
        realm.inspect_object(&reason).unwrap().prototype(),
        Some(&expected)
    );
    realm
        .eval("pair=undefined;p=undefined;reason=undefined;")
        .unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(realm.objects.environment(&environment).is_err());
    assert!(realm.objects.inspect(&expected).is_err());
}

#[test]
fn foreign_promise_job_preflight_refusals_and_started_aborts_restore_the_caller() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        r#"
        var called=0,effects=0;
        function then(resolve){called++;resolve(7);}
        var loop=function(){try{for(;;){}}finally{effects++;}};
        then
    "#,
    );
    expose_child(&mut realm, &environment, function);
    realm
        .eval("var answer,p=Promise.resolve({then:foreign});p.then(x=>answer=x);")
        .unwrap();
    let count = realm.promise_jobs.len();
    let caller_work = realm.remaining_steps;
    realm.limits.max_steps = Some(0);
    assert!(matches!(
        realm.run_next_promise_job(),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.promise_jobs.len(), count);
    assert_eq!(realm.remaining_steps, caller_work);
    assert_eq!(
        realm.record.global_environment.as_ref(),
        realm.scopes.first()
    );
    realm.limits.max_steps = None;
    realm.limits.max_arguments = Some(1);
    assert!(matches!(
        realm.run_next_promise_job(),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.promise_jobs.len(), count);
    assert_eq!(realm.remaining_steps, caller_work);
    assert_eq!(
        realm.record.global_environment.as_ref(),
        realm.scopes.first()
    );
    assert!(
        realm
            .objects
            .environment(&environment)
            .unwrap()
            .inactive_realm
            .is_some()
    );
    realm.limits.max_arguments = None;
    drain_realm_test_jobs(&mut realm);
    assert_eq!(
        realm.eval("answer===7 && child.called===1"),
        Ok(Value::Boolean(true))
    );
    realm.eval("Promise.resolve({then:child.loop});").unwrap();
    realm.limits.max_steps = Some(1000);
    assert!(matches!(
        realm.run_next_promise_job(),
        Ok(crate::PromiseJobOutcome::Reported(Error::Limit { .. }))
    ));
    assert!(realm.promise_jobs.is_empty());
    assert_eq!(
        realm.record.global_environment.as_ref(),
        realm.scopes.first()
    );
    assert!(
        realm
            .objects
            .environment(&environment)
            .unwrap()
            .inactive_realm
            .is_some()
    );
    assert!(!realm.job_active);
    assert_eq!(realm.call_depth, 0);
    assert_eq!(realm.scopes.len(), 1);
    realm.limits.max_steps = None;
    assert_eq!(realm.eval("child.effects"), Ok(Value::Number(0.0)));
}

#[test]
fn suspended_async_generator_returns_await_with_the_source_realms_promise() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        r#"
        var GP=Object.getPrototypeOf(async function*(){}.prototype),cleanup;
        var gen=(async function*(){try{yield 1;}finally{cleanup=[];}})();
        function noop(){}noop
    "#,
    );
    expose_child(&mut realm, &environment, function);
    realm.eval(r#"
        let GP=Object.getPrototypeOf(async function*(){}.prototype),results=[],errors=[],gets=0,cleanup,effects=0;
        let local=(async function*(){try{yield 1;}finally{cleanup=[];}})();
        child.GP.next.call(local);GP.next.call(child.gen);
    "#).unwrap();
    drain_realm_test_jobs(&mut realm);
    assert_eq!(realm.eval(r#"
        let p=Promise.resolve(7),q=child.Promise.resolve(8);
        for(let v of [p,q])Object.defineProperty(v,'then',{get(){gets++;throw 99;}});
        let a=child.GP.return.call(local,p),b=GP.return.call(child.gen,q);
        a.then(r=>results.push(r),e=>errors.push(e));b.then(r=>results.push(r),e=>errors.push(e));
        Object.getPrototypeOf(a)===child.Promise.prototype && Object.getPrototypeOf(b)===Promise.prototype
    "#),Ok(Value::Boolean(true)));
    drain_realm_test_jobs(&mut realm);
    assert_eq!(realm.eval("gets===0 && errors.length===0 && results.length===2 && results.every(r=>r.done) && results.map(r=>r.value).sort().join(',')==='7,8' && Object.getPrototypeOf(cleanup)===Array.prototype && Object.getPrototypeOf(child.cleanup)===child.Array.prototype"),Ok(Value::Boolean(true)));
    realm
        .eval("let aborting=(async function*(){yield 1;})();child.GP.next.call(aborting);")
        .unwrap();
    drain_realm_test_jobs(&mut realm);
    realm.limits.max_steps = Some(1000);
    assert!(matches!(realm.eval("let stuck=Promise.resolve(1);Object.defineProperty(stuck,'constructor',{get(){for(;;){}}});try{child.GP.return.call(aborting,stuck);}catch(e){effects++;}finally{effects++;}"),Err(Error::Limit{..})));
    assert_eq!(
        realm.record.global_environment.as_ref(),
        realm.scopes.first()
    );
    assert_eq!(realm.call_depth, 0);
    assert_eq!(realm.scopes.len(), 1);
    assert!(
        realm
            .objects
            .environment(&environment)
            .unwrap()
            .inactive_realm
            .is_some()
    );
    realm.limits.max_steps = None;
    assert_eq!(realm.eval("effects"), Ok(Value::Number(0.0)));
}

#[test]
fn initial_and_completed_async_generator_returns_keep_the_method_realm() {
    let mut realm = Realm::default();
    let (environment, function) = child_realm_fixture(
        &mut realm,
        "var GP=Object.getPrototypeOf(async function*(){}.prototype);function noop(){}noop",
    );
    expose_child(&mut realm, &environment, function);
    realm
        .eval(
            r#"
        let values=[],errors=[],body=0,gets=0;
        let fresh=(async function*(){body++;})(),finished=(async function*(){})();
        finished.next();
        let own=child.Promise.resolve(7),other=Promise.resolve(8);
        Object.defineProperty(own,'then',{get(){gets++;throw 11;}});
        Object.defineProperty(other,'then',{get(){gets++;throw 99;}});
        child.GP.return.call(fresh,own).then(r=>values.push(r),e=>errors.push(e));
        child.GP.return.call(finished,own).then(r=>values.push(r),e=>errors.push(e));
    "#,
        )
        .unwrap();
    drain_realm_test_jobs(&mut realm);
    assert_eq!(realm.eval("body===0 && gets===0 && errors.length===0 && values.length===2 && values.every(r=>r.done && r.value===7)"),Ok(Value::Boolean(true)));
    realm
        .eval("child.GP.return.call(finished,other).then(r=>values.push(r),e=>errors.push(e));")
        .unwrap();
    drain_realm_test_jobs(&mut realm);
    assert_eq!(realm.eval("body===0 && gets===1 && values.length===2 && errors.length===1 && errors[0]===99 && Object.getPrototypeOf([])===Array.prototype"),Ok(Value::Boolean(true)));
    assert_eq!(
        realm.record.global_environment.as_ref(),
        realm.scopes.first()
    );
}
