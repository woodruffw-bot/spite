//! Source proposal for the complete edition-17 Proxy surface (10.5, 28.2).
//! This off-repository draft has not been parsed by Spite, compiled, or executed.

use crate::common::REALM_ENTRIES;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};

const HELPERS: &str = r#"
function assert(value){if(!value)throw 'proxy assertion';}
function expectTypeError(action){
    let caught=false;
    try{action();}catch(error){caught=error instanceof TypeError;}
    assert(caught);
}
"#;

// Public operations and independently specified trap arities. Each row executes
// an actual language operation; this table does not call private Proxy helpers.
const OPERATIONS: &str = r#"
let operations=[
 ['getPrototypeOf',1,(p,r,v,k)=>Reflect.getPrototypeOf(p)],
 ['setPrototypeOf',2,(p,r,v,k)=>Reflect.setPrototypeOf(p,v)],
 ['isExtensible',1,(p,r,v,k)=>Reflect.isExtensible(p)],
 ['preventExtensions',1,(p,r,v,k)=>Reflect.preventExtensions(p)],
 ['getOwnPropertyDescriptor',2,(p,r,v,k)=>Reflect.getOwnPropertyDescriptor(p,k)],
 ['defineProperty',3,(p,r,v,k)=>Reflect.defineProperty(p,k,{value:9})],
 ['has',2,(p,r,v,k)=>Reflect.has(p,k)],
 ['get',3,(p,r,v,k)=>Reflect.get(p,k,r)],
 ['set',4,(p,r,v,k)=>Reflect.set(p,k,9,r)],
 ['deleteProperty',2,(p,r,v,k)=>Reflect.deleteProperty(p,k)],
 ['ownKeys',1,(p,r,v,k)=>Reflect.ownKeys(p)],
 ['apply',3,(p,r,v,k)=>Reflect.apply(p,r,[])],
 ['construct',3,(p,r,v,k)=>Reflect.construct(p,[],Function)]
];
function target(){return function target(){return 7;};}
"#;

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(&format!("{HELPERS}{source}")),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn matrix(source: &str) {
    check(&format!("{OPERATIONS}{source}"));
}

fn log(source: &str) -> String {
    let result = Realm::default().eval(&format!("{HELPERS}{source}"));
    let Ok(Value::String(value)) = result else {
        panic!("Expected order log: {result:?}; {source}")
    };
    value.to_utf8().unwrap()
}

#[test]
fn creation_revocable_metadata_and_new_target_validation_do_not_invoke_object_hooks() {
    check(
        r#"
    let effects='',bad={get prototype(){throw 1;},[Symbol.toPrimitive](){throw 2;}};
    expectTypeError(()=>Proxy((effects+='t',{}),(effects+='h',{})));
    assert(effects==='th');
    for(let primitive of [undefined,null,false,1,1n,'x',Symbol()]){
        expectTypeError(()=>new Proxy(primitive,bad));
        expectTypeError(()=>new Proxy({},primitive));
        expectTypeError(()=>Proxy.revocable(primitive,bad));
    }
    let N=(function(){}).bind(null);
    Object.defineProperty(N,'prototype',{get(){throw 3;}});
    let original={},p=Reflect.construct(Proxy,[original,{}],N);
    assert(Object.getPrototypeOf(p)===Object.prototype);
    let result=Proxy.revocable.call(null,{},{}),revoke=result.revoke;
    assert(Reflect.ownKeys(result).join(',')==='proxy,revoke');
    for(let name of ['proxy','revoke']){
        let d=Object.getOwnPropertyDescriptor(result,name);
        assert(d.writable && d.enumerable && d.configurable);
    }
    assert(revoke.name==='' && revoke.length===0);
    assert(Object.getPrototypeOf(revoke)===Function.prototype);
    assert(!Object.hasOwn(revoke,'prototype'));
    expectTypeError(()=>new revoke());
    assert(revoke.call(Symbol(),1,2)===undefined && revoke()===undefined);
    let revoked=Proxy.revocable(function(){},{});revoked.revoke();
    assert(typeof new Proxy(revoked.proxy,{})==='function');
    let intercepted=new Proxy(revoked.proxy,{apply(){return 7;},construct(){return {n:9};}});
    assert(intercepted()===7 && new intercepted().n===9);
    let handler=Proxy.revocable({},{});handler.revoke();
    let wrapped=new Proxy({},handler.proxy);
    expectTypeError(()=>wrapped.x);
    !Object.hasOwn(Proxy,'prototype') && Proxy.length===2 && Proxy.revocable.length===2
    "#,
    );
}

#[test]
fn every_missing_or_null_trap_forwards_and_every_noncallable_or_abrupt_trap_stops() {
    matrix(
        r#"
    for(let [name,arity,operation] of operations){
        for(let absent of [undefined,null]){
            let t=target();t.x=1;let handler={[name]:absent},p=new Proxy(t,handler);
            // Forward all thirteen operations; preventExtensions is last for its
            // fresh target and construct uses an explicit ordinary newTarget.
            let receiver={},value=operation(p,receiver,null,'x');
            if(name==='getPrototypeOf')assert(value===Function.prototype);
            if(name==='setPrototypeOf')assert(value && Object.getPrototypeOf(t)===null);
            if(name==='isExtensible')assert(value===true);
            if(name==='getOwnPropertyDescriptor')assert(value.value===1 && value.writable && value.enumerable && value.configurable);
            if(name==='has')assert(value===true);
            if(name==='get')assert(value===1);
            if(name==='set')assert(value && receiver.x===9 && t.x===1);
            if(name==='defineProperty')assert(value && t.x===9);
            if(name==='deleteProperty')assert(value && !Object.hasOwn(t,'x'));
            if(name==='ownKeys')assert(Array.isArray(value) && value.includes('x') && value.includes('prototype'));
            if(name==='apply')assert(value===7);
            if(name==='construct')assert(typeof value==='object' && Object.getPrototypeOf(value)===Function.prototype);
            if(name==='preventExtensions')assert(value && !Object.isExtensible(t));
        }
        expectTypeError(()=>operation(new Proxy(target(),{[name]:1}),{},null,'x'));
        let marker={},handler={};
        Object.defineProperty(handler,name,{get(){throw marker;}});
        let caught=false;
        try{operation(new Proxy(target(),handler),{},null,'x');}catch(e){caught=e===marker;}
        assert(caught);
    }
    true
    "#,
    );
}

#[test]
fn all_thirteen_traps_receive_handler_this_and_exact_uncoerced_arguments() {
    matrix(
        r#"
    for(let key of [Symbol('key'),'\uD800']){
        for(let [name,arity,operation] of operations){
            let t=target(),receiver=37,prototype={},handler={},calls=0;t[key]=1;
            handler[name]=function(...args){
                calls++;assert(this===handler && args.length===arity && args[0]===t);
                if(['getOwnPropertyDescriptor','defineProperty','has','get','set','deleteProperty'].includes(name))assert(args[1]===key);
                if(name==='setPrototypeOf')assert(args[1]===prototype);
                if(name==='get')assert(args[2]===receiver);
                if(name==='set')assert(args[2]===9 && args[3]===receiver);
                if(name==='defineProperty')assert(Reflect.ownKeys(args[2]).join(',')==='value' && args[2].value===9);
                if(name==='apply')assert(args[1]===receiver && Array.isArray(args[2]) && args[2].length===0);
                if(name==='construct')assert(Array.isArray(args[1]) && args[1].length===0 && args[2]===Function);
                if(name==='getPrototypeOf')return prototype;
                if(name==='getOwnPropertyDescriptor')return Object.getOwnPropertyDescriptor(t,key);
                if(name==='ownKeys')return Reflect.ownKeys(t);
                if(name==='construct')return {};
                if(name==='get' || name==='apply')return 7;
                return name!=='preventExtensions' && name!=='deleteProperty';
            };
            operation(new Proxy(t,handler),receiver,prototype,key);
            assert(calls===1);
        }
    }
    true
    "#,
    );
}

#[test]
fn prototype_traps_validate_results_before_target_queries_and_preserve_false_early_exit() {
    check(
        r#"
    let queries=0,base={},inner=new Proxy(base,{
        isExtensible(t){queries++;return Reflect.isExtensible(t);},
        getPrototypeOf(t){queries++;return Reflect.getPrototypeOf(t);}
    });
    expectTypeError(()=>Object.getPrototypeOf(new Proxy(inner,{getPrototypeOf(){return 1;}})));
    assert(queries===0);
    assert(!Reflect.setPrototypeOf(new Proxy(inner,{setPrototypeOf(){return 0;}}),null));
    assert(queries===0);
    let alternate={};
    assert(Reflect.getPrototypeOf(new Proxy(inner,{getPrototypeOf(){return alternate;}}))===alternate);
    assert(queries===1);
    Object.preventExtensions(base);queries=0;
    let good=new Proxy(inner,{getPrototypeOf(){return Object.prototype;},setPrototypeOf(){return {};}});
    assert(Reflect.getPrototypeOf(good)===Object.prototype);
    assert(Reflect.setPrototypeOf(good,Object.prototype));
    assert(queries===4);
    expectTypeError(()=>Reflect.setPrototypeOf(good,null));
    expectTypeError(()=>Object.getPrototypeOf(new Proxy(base,{getPrototypeOf(){return null;}})));
    true
    "#,
    );
}

#[test]
fn extensibility_boolean_results_are_checked_live_and_prevent_false_skips_queries() {
    check(
        r#"
    let queries=0,base={},inner=new Proxy(base,{isExtensible(t){queries++;return Reflect.isExtensible(t);}});
    expectTypeError(()=>Reflect.isExtensible(new Proxy(inner,{isExtensible(){return false;}})));
    assert(queries===1);
    let truthy={valueOf(){throw 1;},toString(){throw 2;}};
    assert(Reflect.isExtensible(new Proxy(inner,{isExtensible(){return truthy;}})));
    queries=0;
    assert(!Reflect.preventExtensions(new Proxy(inner,{preventExtensions(){return false;}})) && queries===0);
    expectTypeError(()=>Reflect.preventExtensions(new Proxy(inner,{preventExtensions(){return true;}})));
    assert(queries===1);
    let p=new Proxy(inner,{preventExtensions(){Object.preventExtensions(base);return truthy;}});
    assert(Reflect.preventExtensions(p) && !Reflect.isExtensible(p));
    expectTypeError(()=>Reflect.isExtensible(new Proxy(base,{isExtensible(){return true;}})));
    true
    "#,
    );
}

#[test]
fn own_descriptor_traps_enforce_absence_compatibility_and_nonconfigurable_restrictions() {
    check(
        r#"
    let observed=0,inner=new Proxy({},{getOwnPropertyDescriptor(){observed++;throw 1;}});
    expectTypeError(()=>Object.getOwnPropertyDescriptor(new Proxy(inner,{getOwnPropertyDescriptor(){return null;}}),'x'));
    assert(observed===0);
    let absentTarget=new Proxy({},{getOwnPropertyDescriptor(){return undefined;},isExtensible(){throw 2;}});
    let absent=new Proxy(absentTarget,{getOwnPropertyDescriptor(){return undefined;}});
    assert(Object.getOwnPropertyDescriptor(absent,'x')===undefined);
    let fixedBase={};Object.defineProperty(fixedBase,'x',{value:1});
    let fixedTarget=new Proxy(fixedBase,{
        getOwnPropertyDescriptor(t,k){return Reflect.getOwnPropertyDescriptor(t,k);},
        isExtensible(){throw 3;}
    });
    expectTypeError(()=>Object.getOwnPropertyDescriptor(new Proxy(fixedTarget,{getOwnPropertyDescriptor(){return undefined;}}),'x'));
    for(let setup of [
        t=>Object.defineProperty(t,'x',{value:1,configurable:false}),
        t=>{t.x=1;Object.preventExtensions(t);}
    ]){
        let t={};setup(t);
        expectTypeError(()=>Object.getOwnPropertyDescriptor(new Proxy(t,{getOwnPropertyDescriptor(){return undefined;}}),'x'));
    }
    let fixed={};Object.preventExtensions(fixed);
    expectTypeError(()=>Object.getOwnPropertyDescriptor(new Proxy(fixed,{getOwnPropertyDescriptor(){return {value:1,configurable:true};}}),'x'));
    for(let t of [{},{x:1}]){
        expectTypeError(()=>Object.getOwnPropertyDescriptor(new Proxy(t,{getOwnPropertyDescriptor(){return {value:1};}}),'x'));
    }
    let writable={};Object.defineProperty(writable,'x',{value:1,writable:true});
    expectTypeError(()=>Object.getOwnPropertyDescriptor(new Proxy(writable,{getOwnPropertyDescriptor(){return {value:1,writable:false};}}),'x'));
    let frozen={};Object.defineProperty(frozen,'x',{value:NaN});
    assert(Number.isNaN(Object.getOwnPropertyDescriptor(new Proxy(frozen,{getOwnPropertyDescriptor(){return {value:NaN};}}),'x').value));
    expectTypeError(()=>Object.getOwnPropertyDescriptor(new Proxy(frozen,{getOwnPropertyDescriptor(){return {value:0};}}),'x'));
    let visible={x:1};Object.preventExtensions(visible);
    assert(Object.getOwnPropertyDescriptor(new Proxy(visible,{getOwnPropertyDescriptor(){return {value:9,writable:true,enumerable:true,configurable:true};}}),'x').value===9);
    true
    "#,
    );
}

#[test]
fn own_descriptor_nested_conversion_uses_captured_target_facts_and_ordered_has_get() {
    // 10.5.5 observes targetDesc and extensibility before ToPropertyDescriptor.
    insta::assert_snapshot!(log(r#"
    let rows=[],base={},target=new Proxy(base,{
        getOwnPropertyDescriptor(){rows.push('target:desc');return undefined;},
        isExtensible(t){rows.push('target:ext');return Reflect.isExtensible(t);}
    });
    let descriptor=new Proxy({configurable:true,value:7},{
        has(t,k){rows.push('has:'+k);return Reflect.has(t,k);},
        get(t,k,r){rows.push('get:'+k);if(k==='configurable')Object.preventExtensions(base);return Reflect.get(t,k,r);}
    });
    let p=new Proxy(target,{getOwnPropertyDescriptor(){rows.push('trap');return descriptor;}});
    let d=Object.getOwnPropertyDescriptor(p,'x');
    assert(d.value===7 && d.configurable && !d.enumerable && !d.writable);
    assert(!Object.isExtensible(base) && !Object.hasOwn(base,'x'));
    rows.join('|')
    "#), @"trap|target:desc|target:ext|has:enumerable|has:configurable|get:configurable|has:value|get:value|has:writable|has:get|has:set");
}

#[test]
fn descriptor_conversion_validates_get_before_set_and_mixed_kind_after_both() {
    check(
        r#"
    for(let mixed of [false,true]){
        let rows=[],d={configurable:true,value:7,get:mixed?undefined:1,set:undefined};
        let descriptor=new Proxy(d,{
            has(t,k){rows.push('has:'+k);return Reflect.has(t,k);},
            get(t,k,r){rows.push('get:'+k);return Reflect.get(t,k,r);}
        });
        expectTypeError(()=>Object.getOwnPropertyDescriptor(new Proxy({},{getOwnPropertyDescriptor(){return descriptor;}}),'x'));
        assert(rows.includes('get:get'));
        if(mixed)assert(rows.slice(-2).join(',')==='has:set,get:set');
        else assert(rows[rows.length-1]==='get:get' && !rows.includes('has:set'));
    }
    true
    "#,
    );
}

#[test]
fn define_trap_receives_only_present_descriptor_fields_and_false_skips_target_checks() {
    check(
        r#"
    let getter=function(){},setter=function(v){},key=Symbol(),seen=[];
    let target=new Proxy({},{getOwnPropertyDescriptor(){throw 1;},isExtensible(){throw 2;}});
    let handler={defineProperty(t,k,d){
        assert(t===target && k===key && this===handler);
        seen.push(d);return false;
    }},p=new Proxy(target,handler);
    assert(!Reflect.defineProperty(p,key,{value:7}));
    assert(Reflect.ownKeys(seen[0]).join(',')==='value' && seen[0].value===7);
    assert(!Reflect.defineProperty(p,key,{get:getter,set:setter,configurable:true}));
    assert(Reflect.ownKeys(seen[1]).length===3 && !Object.hasOwn(seen[1],'value') && !Object.hasOwn(seen[1],'enumerable') && !Object.hasOwn(seen[1],'writable'));
    assert(seen[1].get===getter && seen[1].set===setter && seen[1].configurable);
    expectTypeError(()=>Object.defineProperty(p,key,{value:7}));
    true
    "#,
    );
}

#[test]
fn define_trap_success_checks_live_target_without_applying_a_fake_descriptor() {
    check(
        r#"
    let rows=[],observedTarget=new Proxy({}, {
        getOwnPropertyDescriptor(){rows.push('own');return undefined;},
        isExtensible(){rows.push('ext');return true;}
    });
    assert(Reflect.defineProperty(new Proxy(observedTarget,{defineProperty(){return true;}}),'x',{value:7,configurable:true}));
    assert(rows.join(',')==='own,ext');
    let t={},p=new Proxy(t,{defineProperty(){return true;}});
    assert(Reflect.defineProperty(p,'x',{value:7,configurable:true}) && !Object.hasOwn(t,'x'));
    expectTypeError(()=>Reflect.defineProperty(p,'x',{value:7,configurable:false}));
    Object.preventExtensions(t);
    expectTypeError(()=>Reflect.defineProperty(p,'x',{value:7,configurable:true}));
    let configurable={x:1};
    expectTypeError(()=>Reflect.defineProperty(new Proxy(configurable,{defineProperty(){return true;}}),'x',{configurable:false}));
    let fixed={};Object.defineProperty(fixed,'x',{value:NaN,writable:true});
    expectTypeError(()=>Reflect.defineProperty(new Proxy(fixed,{defineProperty(){return true;}}),'x',{writable:false}));
    let mutable=new Proxy(fixed,{defineProperty(t,k,d){return Reflect.defineProperty(t,k,d);}});
    assert(Reflect.defineProperty(mutable,'x',{writable:false}));
    assert(Reflect.defineProperty(mutable,'x',{value:NaN}));
    expectTypeError(()=>Reflect.defineProperty(new Proxy(fixed,{defineProperty(){return true;}}),'x',{value:0}));
    let zero={};Object.defineProperty(zero,'x',{value:-0});
    expectTypeError(()=>Reflect.defineProperty(new Proxy(zero,{defineProperty(){return true;}}),'x',{value:0}));
    let accessor={};Object.defineProperty(accessor,'x',{get(){return 1;}});
    expectTypeError(()=>Reflect.defineProperty(new Proxy(accessor,{defineProperty(){return true;}}),'x',{value:1}));
    true
    "#,
    );
}

#[test]
fn has_true_and_missing_false_skip_invariant_queries_but_existing_fixed_keys_cannot_hide() {
    check(
        r#"
    let inner=new Proxy({},{getOwnPropertyDescriptor(){throw 1;}});
    assert(Reflect.has(new Proxy(inner,{has(){return {};}}),'x'));
    let count=0,missing=new Proxy({},{getOwnPropertyDescriptor(){count++;return undefined;},isExtensible(){throw 2;}});
    assert(!Reflect.has(new Proxy(missing,{has(){return false;}}),'x') && count===1);
    let inherited={};Object.defineProperty(inherited,'x',{value:1});
    assert(!Reflect.has(new Proxy(Object.create(inherited),{has(){return false;}}),'x'));
    let configurable={x:1},p=new Proxy(configurable,{has(){return false;}});
    assert(!('x' in p));Object.preventExtensions(configurable);
    expectTypeError(()=>Reflect.has(p,'x'));
    let fixed={};Object.defineProperty(fixed,'x',{value:1});
    let fixedTarget=new Proxy(fixed,{
        getOwnPropertyDescriptor(t,k){return Reflect.getOwnPropertyDescriptor(t,k);},
        isExtensible(){throw 3;}
    });
    expectTypeError(()=>Reflect.has(new Proxy(fixedTarget,{has(){return false;}}),'x'));
    true
    "#,
    );
}

#[test]
fn get_invariants_use_same_value_and_do_not_invoke_target_getters() {
    check(
        r#"
    let t={};Object.defineProperty(t,'nan',{value:NaN});Object.defineProperty(t,'zero',{value:-0});
    Object.defineProperty(t,'empty',{get:undefined});
    let p=new Proxy(t,{get(t,k){if(k==='nan')return NaN;if(k==='zero')return -0;return undefined;}});
    assert(Number.isNaN(p.nan) && Object.is(p.zero,-0) && p.empty===undefined);
    expectTypeError(()=>new Proxy(t,{get(){return 0;}}).zero);
    expectTypeError(()=>new Proxy(t,{get(){return 1;}}).nan);
    expectTypeError(()=>new Proxy(t,{get(){return 1;}}).empty);
    let calls=0,accessor={get x(){calls++;throw 1;}};
    assert(new Proxy(accessor,{get(){return 9;}}).x===9 && calls===0);
    let changed={},q=new Proxy(changed,{get(t,k){Object.defineProperty(t,k,{value:7});return 8;}});
    expectTypeError(()=>q.x);
    true
    "#,
    );
}

#[test]
fn set_preserves_false_short_circuit_same_value_and_receiver_without_extra_write() {
    check(
        r#"
    let inner=new Proxy({},{getOwnPropertyDescriptor(){throw 1;}});
    assert(!Reflect.set(new Proxy(inner,{set(){return false;}}),'x',1));
    let t={};Object.defineProperty(t,'nan',{value:NaN});Object.defineProperty(t,'zero',{value:-0});
    Object.defineProperty(t,'empty',{set:undefined});
    let p=new Proxy(t,{set(){return true;}});
    assert(Reflect.set(p,'nan',NaN) && Reflect.set(p,'zero',-0));
    expectTypeError(()=>Reflect.set(p,'zero',0));
    expectTypeError(()=>Reflect.set(p,'nan',0));
    expectTypeError(()=>Reflect.set(p,'empty',undefined));
    let receiver=Object.preventExtensions({}),seen=0;
    let virtual=new Proxy({},{set(t,k,v,r){seen++;assert(r===receiver);return {};}});
    assert(Reflect.set(virtual,'x',7,receiver) && seen===1 && !Object.hasOwn(receiver,'x'));
    let denied=new Proxy({},{set(){return false;}});
    assert((denied.x=7)===7);
    expectTypeError(()=>{ 'use strict'; denied.x=7; });
    true
    "#,
    );
}

#[test]
fn delete_checks_only_live_own_state_and_keeps_all_false_and_absent_early_exits() {
    check(
        r#"
    let inner=new Proxy({},{getOwnPropertyDescriptor(){throw 1;}});
    assert(!Reflect.deleteProperty(new Proxy(inner,{deleteProperty(){return false;}}),'x'));
    let missing=new Proxy({},{getOwnPropertyDescriptor(){return undefined;},isExtensible(){throw 2;}});
    assert(Reflect.deleteProperty(new Proxy(missing,{deleteProperty(){return true;}}),'x'));
    let fixed={};Object.defineProperty(fixed,'x',{value:1});
    let fixedTarget=new Proxy(fixed,{
        getOwnPropertyDescriptor(t,k){return Reflect.getOwnPropertyDescriptor(t,k);},
        isExtensible(){throw 3;}
    });
    expectTypeError(()=>Reflect.deleteProperty(new Proxy(fixedTarget,{deleteProperty(){return true;}}),'x'));
    let t={x:1},p=new Proxy(t,{deleteProperty(){return true;}});
    assert(Reflect.deleteProperty(p,'x') && t.x===1);
    Object.preventExtensions(t);expectTypeError(()=>Reflect.deleteProperty(p,'x'));
    let removed={x:1};Object.preventExtensions(removed);
    assert(Reflect.deleteProperty(new Proxy(removed,{deleteProperty(t,k){return delete t[k];}}),'x'));
    let denied=new Proxy({},{deleteProperty(){return false;}});
    assert(delete denied.x===false);
    expectTypeError(()=>{ 'use strict'; delete denied.x; });
    true
    "#,
    );
}

#[test]
fn own_keys_preserves_string_symbol_order_and_rejects_nonkeys_without_coercion() {
    check(
        r#"
    let a=Symbol('same'),b=Symbol('same'),keys=[b,'2','\uD800',a,'0'];
    let p=new Proxy({},{ownKeys(){return keys;}}),actual=Reflect.ownKeys(p);
    assert(actual.length===5 && actual.every((v,i)=>v===keys[i]));
    assert(Object.getOwnPropertyNames(p).join('|')==='2|\uD800|0');
    assert(Object.getOwnPropertySymbols(p)[0]===b && Object.getOwnPropertySymbols(p)[1]===a);
    let poisonTarget=new Proxy({},{isExtensible(){throw 1;}});
    for(let result of [undefined,null,1,'x',Symbol()])expectTypeError(()=>Reflect.ownKeys(new Proxy(poisonTarget,{ownKeys(){return result;}})));
    let coerced=0,key={toString(){coerced++;return 'x';},[Symbol.toPrimitive](){coerced++;return 'x';}};
    for(let value of [undefined,null,1,1n,false,key])expectTypeError(()=>Reflect.ownKeys(new Proxy(poisonTarget,{ownKeys(){return [value];}})));
    expectTypeError(()=>Reflect.ownKeys(new Proxy({},{ownKeys(){return Array(1);}})));
    expectTypeError(()=>Reflect.ownKeys(new Proxy({},{ownKeys(){return [a,a];}})));
    expectTypeError(()=>Reflect.ownKeys(new Proxy({},{ownKeys(){return ['x','x'];}})));
    coerced===0
    "#,
    );
}

#[test]
fn own_keys_finishes_nested_array_like_gets_before_duplicate_detection() {
    // 10.5.11 finishes CreateListFromArrayLike before checking duplicates.
    insta::assert_snapshot!(log(r#"
    let rows=[],target=new Proxy({},{isExtensible(){rows.push('unexpected:target');return true;}});
    let list=new Proxy({0:'x',1:'x',2:'later',length:{valueOf(){rows.push('number');return 3;}}},{
        get(t,k,r){rows.push('get:'+k);return Reflect.get(t,k,r);}
    });
    let p=new Proxy(target,{ownKeys(){rows.push('trap');return list;}});
    expectTypeError(()=>Reflect.ownKeys(p));rows.push('TypeError');
    rows.join('|')
    "#), @"trap|get:length|number|get:0|get:1|get:2|TypeError");
    check(
        r#"
    let later=0,list={length:2,0:1,get 1(){later++;return 'x';}};
    expectTypeError(()=>Reflect.ownKeys(new Proxy({},{ownKeys(){return list;}})));
    later===0
    "#,
    );
}

#[test]
fn own_keys_walks_all_target_descriptors_then_enforces_fixed_key_membership() {
    check(
        r#"
    let rows=[],base={a:1,b:2},target=new Proxy(base,{
        isExtensible(t){rows.push('ext');return Reflect.isExtensible(t);},
        ownKeys(t){rows.push('keys');return Reflect.ownKeys(t);},
        getOwnPropertyDescriptor(t,k){rows.push('desc:'+k);return Reflect.getOwnPropertyDescriptor(t,k);}
    });
    let p=new Proxy(target,{ownKeys(){rows.push('outer');return ['b','a','extra'];}});
    assert(Reflect.ownKeys(p).join(',')==='b,a,extra');
    assert(rows.join('|')==='outer|ext|keys|desc:a|desc:b');
    let symbol=Symbol(),t={a:1};Object.defineProperty(t,symbol,{value:2});
    expectTypeError(()=>Reflect.ownKeys(new Proxy(t,{ownKeys(){return ['a'];}})));
    assert(Reflect.ownKeys(new Proxy(t,{ownKeys(){return [symbol,'extra'];}}))[0]===symbol);
    Object.preventExtensions(t);
    for(let keys of [[symbol],[symbol,'a','extra']]){
        expectTypeError(()=>Reflect.ownKeys(new Proxy(t,{ownKeys(){return keys;}})));
    }
    let exact=Reflect.ownKeys(new Proxy(t,{ownKeys(){return [symbol,'a'];}}));
    exact.length===2 && exact[0]===symbol && exact[1]==='a'
    "#,
    );
}

#[test]
fn revoked_operations_throw_before_trap_lookup_but_identity_and_capabilities_remain() {
    matrix(
        r#"
    for(let [name,arity,operation] of operations){
        let reads=0,handler={};Object.defineProperty(handler,name,{get(){reads++;throw 1;}});
        let r=Proxy.revocable(target(),handler),p=r.proxy;r.revoke();
        expectTypeError(()=>operation(p,{},null,'x'));assert(reads===0);
        assert(typeof p==='function' && Object.is(p,p));
        assert(Function.prototype.toString.call(p).includes('[native code]'));
        let map=new WeakMap([[p,7]]),set=new WeakSet([p]);
        assert(map.get(p)===7 && set.has(p));
        expectTypeError(()=>Array.isArray(p));
    }
    let ordinary=Proxy.revocable({},{});ordinary.revoke();
    typeof ordinary.proxy==='object'
    "#,
    );
}

#[test]
fn revocation_during_trap_getter_uses_the_already_captured_pair_for_all_methods() {
    matrix(
        r#"
    for(let [name,arity,operation] of operations){
        let t=target(),handler={},reads=0,calls=0,r;t.x=1;
        Object.defineProperty(handler,name,{get(){
            reads++;r.revoke();
            return function(...args){
                calls++;assert(this===handler && args[0]===t);
                if(name==='getPrototypeOf')return null;
                if(name==='getOwnPropertyDescriptor')return Object.getOwnPropertyDescriptor(t,'x');
                if(name==='ownKeys')return Reflect.ownKeys(t);
                if(name==='construct')return {};
                if(name==='get' || name==='apply')return 7;
                return name!=='preventExtensions' && name!=='deleteProperty';
            };
        }});
        r=Proxy.revocable(t,handler);
        operation(r.proxy,{},null,'x');
        assert(reads===1 && calls===1);
        expectTypeError(()=>operation(r.proxy,{},null,'x'));
        assert(reads===1);
    }
    let inner=Proxy.revocable({},{}),outer=new Proxy(inner.proxy,{get(){inner.revoke();return 7;}});
    expectTypeError(()=>outer.x);
    true
    "#,
    );
}

#[test]
fn prototype_get_set_and_distinct_proxy_receivers_use_the_correct_internal_operation() {
    check(
        r#"
    let receiver,seen=[],p=new Proxy({}, {
        get(t,k,r){seen.push('get');assert(r===receiver);return 7;},
        set(t,k,v,r){seen.push('set');assert(r===receiver && v===9);return true;},
        getOwnPropertyDescriptor(){throw 1;}
    });
    receiver=Object.create(p);
    assert(receiver.x===7);receiver.x=9;
    assert(seen.join(',')==='get,set' && !Object.hasOwn(receiver,'x'));
    let base={x:2},target={x:1},events=[];
    let destination=new Proxy(base,{
        getOwnPropertyDescriptor(t,k){events.push('own:'+k);return Reflect.getOwnPropertyDescriptor(t,k);},
        defineProperty(t,k,d){events.push('define:'+Reflect.ownKeys(d).join(','));return Reflect.defineProperty(t,k,d);}
    });
    assert(Reflect.set(target,'x',7,destination));
    assert(events.join('|')==='own:x|define:value' && target.x===1 && base.x===7);
    let primitive=Symbol(),q=new Proxy({},{get(t,k,r){return r;},set(t,k,v,r){return r===primitive;}});
    assert(Reflect.get(q,'x',primitive)===primitive && Reflect.set(q,'x',1,primitive));
    let home={read(){return super.x;},write(){super.x=9;}},expected=home;
    Object.setPrototypeOf(home,new Proxy({}, {
        get(t,k,r){assert(r===expected);return 7;},
        set(t,k,v,r){assert(r===expected && v===9);return true;}
    }));
    home.read()===7 && home.write()===undefined
    "#,
    );
}

#[test]
fn prototype_consumers_preserve_revoked_identity_short_circuits_and_for_in_traps() {
    check(
        r#"
    let pair=Proxy.revocable(function(){},{}),p=pair.proxy,child=Object.create(p);
    pair.revoke();
    assert(Object.prototype.isPrototypeOf.call(p,child));
    assert(Function.prototype[Symbol.hasInstance].call(p,1)===false);
    expectTypeError(()=>1 instanceof p);
    let ordinary={};assert(Reflect.setPrototypeOf(ordinary,p));
    assert(Object.getPrototypeOf(ordinary)===p);
    let gets=0,a={},candidate=new Proxy(a,{getPrototypeOf(){gets++;throw 1;}});
    assert(Reflect.setPrototypeOf(a,candidate) && gets===0);
    let proto={b:2},base={};Object.defineProperty(base,'a',{enumerable:true,configurable:true,get(){throw 2;}});
    let rows=[],proxy=new Proxy(base,{
        ownKeys(){rows.push('keys');return ['a'];},
        getOwnPropertyDescriptor(t,k){rows.push('own:'+k);return Reflect.getOwnPropertyDescriptor(t,k);},
        getPrototypeOf(){rows.push('proto');return proto;}
    }),keys=[];
    for(let key in proxy)keys.push(key);
    keys.join(',')==='a,b' && rows.includes('keys') && rows.includes('own:a') && rows.includes('proto')
    "#,
    );
}

#[test]
fn bind_order_and_class_super_dispatch_observe_proxy_prototypes_and_new_target() {
    insta::assert_snapshot!(log(r#"
    let rows=[],sum=function sum(a,b){return a+b;},p=new Proxy(sum,{
        getPrototypeOf(t){rows.push('proto');return Reflect.getPrototypeOf(t);},
        getOwnPropertyDescriptor(t,k){rows.push('own:'+k);return Reflect.getOwnPropertyDescriptor(t,k);},
        get(t,k,r){rows.push('get:'+k);return Reflect.get(t,k,r);}
    });
    let bound=Function.prototype.bind.call(p,null,1);
    assert(bound.length===1 && bound.name==='bound sum' && bound(2)===3);
    rows.join('|')
    "#), @"proto|own:length|get:length|get:name");
    check(
        r#"
    let newTarget,receiver;
    class Base{constructor(v){this.value=v;newTarget=new.target;}static x=7;}
    let p=new Proxy(Base,{
        construct(t,args,N){assert(N===Derived);return Reflect.construct(t,args,N);},
        get(t,k,r){if(k==='x')receiver=r;return Reflect.get(t,k,r);}
    });
    class Derived extends p{static read(){return super.x;}}
    let value=new Derived(9);
    value.value===9 && value instanceof Derived && value instanceof Base && newTarget===Derived && Derived.read()===7 && receiver===Derived
    "#,
    );
}

#[test]
fn abstract_is_array_unwraps_all_consumers_while_direct_internal_brands_do_not() {
    check(
        r#"
    let p=new Proxy([1,2],{});
    assert(Array.isArray(p));
    assert(Array.prototype.map.call(p,x=>x*2).join(',')==='2,4');
    assert([].concat(p).join(',')==='1,2');
    assert([p].flat().join(',')==='1,2');
    assert(Object.prototype.toString.call(p)==='[object Array]');
    assert(JSON.stringify({a:1,b:2},new Proxy(['b'],{}))==='{"b":2}');
    assert(JSON.stringify(p)==='[1,2]');
    let effects=0,callable=Proxy.revocable(function(k,v){return v;},{});callable.revoke();
    expectTypeError(()=>JSON.stringify({toJSON(){effects++;return 1;}},callable.proxy));
    assert(effects===1);
    let noncallable=Proxy.revocable([],{});noncallable.revoke();effects=0;
    expectTypeError(()=>JSON.stringify({toJSON(){effects++;return 1;}},noncallable.proxy));
    assert(effects===0);
    let seen=[];
    let parsed=JSON.parse('{"first":0,"later":[0]}',function(k,v){
        if(k==='first')this.later=p;
        if(k==='0' || k==='1')seen.push(k);
        return v;
    });
    assert(parsed.later===p && seen.join(',')==='0,1');
    let view=new Proxy(new Uint8Array(1),{}),data=new Proxy(new DataView(new ArrayBuffer(1)),{});
    assert(!ArrayBuffer.isView(view) && !ArrayBuffer.isView(data));
    expectTypeError(()=>Uint8Array.prototype.set.call(view,[]));
    expectTypeError(()=>DataView.prototype.getUint8.call(data,0));
    expectTypeError(()=>Map.prototype.get.call(new Proxy(new Map(),{}),{}));
    expectTypeError(()=>Set.prototype.add.call(new Proxy(new Set(),{}),{}));
    expectTypeError(()=>WeakMap.prototype.has.call(new Proxy(new WeakMap(),{}),{}));
    expectTypeError(()=>Number.prototype.valueOf.call(new Proxy(new Number(1),{})));
    let raw=new Proxy(JSON.rawJSON('1'),{});
    assert(!JSON.isRawJSON(raw) && JSON.stringify(raw)==='{"rawJSON":"1"}');
    assert(JSON.stringify(new Proxy(new Number(1),{}))==='{}');
    class C{#x=7;read(){return this.#x;}}
    expectTypeError(()=>C.prototype.read.call(new Proxy(new C(),{})));
    let target={},privateProxy=new Proxy(target,{});
    class Returning{constructor(){return privateProxy;}}
    class Branded extends Returning{#x=9;static read(value){return value.#x;}}
    assert(new Branded()===privateProxy && Branded.read(privateProxy)===9);
    expectTypeError(()=>Branded.read(target));
    true
    "#,
    );
    check(
        r#"
    for(let array of [false,true]){
        let keys=array?['0','1']:['keep','remove'];
        for(let abrupt of ['', 'define', 'delete']){
            let target=array?[1,2]:{keep:1,remove:2},events=[],marker={};
            let p=new Proxy(target,{
                defineProperty(t,k,d){
                    assert(t===target && k===keys[0]);
                    assert(Reflect.ownKeys(d).length===4);
                    for(let field of ['value','writable','enumerable','configurable']){
                        assert(Object.hasOwn(d,field));
                    }
                    assert(d.value===9 && d.writable===true && d.enumerable===true && d.configurable===true);
                    events.push('define');
                    if(abrupt==='define')throw marker;
                    return false;
                },
                deleteProperty(t,k){
                    assert(t===target && k===keys[1]);
                    events.push('delete');
                    if(abrupt==='delete')throw marker;
                    return false;
                },
                set(){throw 'JSON reviver must not invoke set';}
            });
            let caught=false,parsed;
            try{
                parsed=JSON.parse('{"first":0,"later":{}}',function(k,v){
                    if(k==='first')this.later=p;
                    if(this===p && k===keys[0])return 9;
                    if(this===p && k===keys[1])return undefined;
                    return v;
                });
            }catch(error){assert(error===marker);caught=true;}
            assert(caught===(abrupt!==''));
            assert(events.join('|')===(abrupt==='define'?'define':'define|delete'));
            assert(target[keys[0]]===1 && target[keys[1]]===2);
            if(!caught)assert(parsed.later===p);
        }
    }
    true
    "#,
    );
}

#[test]
fn apply_traps_get_fresh_argument_arrays_original_this_and_normal_argument_evaluation() {
    check(
        r#"
    let target=function(){throw 1;},receiver=Symbol(),arrays=[],object={},events=[],handler={
        get apply(){events.push('trap:get');return function(t,r,args){
            assert(this===handler && t===target && r===receiver);
            assert(Array.isArray(args) && Object.getPrototypeOf(args)===Array.prototype);
            assert(args.length===2 && args[0]===object && args[1]===7);
            arrays.push(args);args[0]=null;return object;
        };}
    },p=new Proxy(target,handler);
    let incoming=[(events.push('arg:0'),object),(events.push('arg:1'),7)];
    assert(Reflect.apply(p,receiver,incoming)===object && incoming[0]===object && arrays[0]!==incoming);
    assert(Reflect.apply(p,receiver,incoming)===object && arrays[1]!==incoming && arrays[0]!==arrays[1]);
    assert(events.join('|')==='arg:0|arg:1|trap:get|trap:get');
    let forward=new Proxy(function(a){'use strict';return [this,a];},{apply:null});
    let actual=Reflect.apply(forward,receiver,[object]);
    assert(actual[0]===receiver && actual[1]===object);
    let reads=0,notCallable=new Proxy({},{get apply(){reads++;throw 2;}}),effect=0;
    expectTypeError(()=>notCallable(effect=1));
    effect===1 && reads===0
    "#,
    );
}

#[test]
fn construct_traps_preserve_new_target_and_validate_only_the_returned_object() {
    check(
        r#"
    let target=function(){throw 1;},result={},arrays=[],incoming=[7],N=(function(){}).bind(null);
    Object.defineProperty(N,'prototype',{get(){throw 2;}});
    let handler={construct(t,args,newTarget){
        assert(this===handler && t===target && newTarget===N);
        assert(Array.isArray(args) && args[0]===7);arrays.push(args);return result;
    }},p=new Proxy(target,handler);
    assert(Reflect.construct(p,incoming,N)===result && Reflect.construct(p,incoming,N)===result);
    assert(arrays[0]!==incoming && arrays[1]!==incoming && arrays[0]!==arrays[1]);
    for(let value of [undefined,null,false,7,1n,'x',Symbol()]){
        expectTypeError(()=>new (new Proxy(function(){},{construct(){return value;}}))());
    }
    let observed,proto={},F=function(v){this.value=v;observed=new.target;},Other=function(){};
    Other.prototype=proto;
    let value=Reflect.construct(new Proxy(F,{construct:undefined}),[9],Other);
    assert(value.value===9 && observed===Other && Object.getPrototypeOf(value)===proto);
    let reads=0,notConstructor=new Proxy(()=>0,{get construct(){reads++;throw 3;}}),effects=0;
    expectTypeError(()=>new notConstructor(effects=1));
    effects===1 && reads===0
    "#,
    );
}

#[test]
fn rooted_proxy_keeps_both_slots_alive_and_dropping_the_root_reclaims_the_graph() {
    let mut realm = Realm::default();
    let values = realm
        .eval(
            r#"(()=>{
        let target={x:7},handler={get(t,k){return t[k];}};
        return [new Proxy(target,handler),target,handler];
    })()"#,
        )
        .unwrap();
    let proxy = realm.read_property(&values, &JsString::from("0")).unwrap();
    let Value::Object(target) = realm.read_property(&values, &JsString::from("1")).unwrap() else {
        panic!("Expected target")
    };
    let Value::Object(handler) = realm.read_property(&values, &JsString::from("2")).unwrap() else {
        panic!("Expected handler")
    };
    let root = realm.root_value(proxy.clone(), usize::MAX).unwrap();
    drop(values);
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(&target).is_ok());
    assert!(realm.inspect_object(&handler).is_ok());
    assert_eq!(
        realm.read_property(&proxy, &JsString::from("x")),
        Ok(Value::Number(7.0))
    );
    drop(root);
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert!(realm.inspect_object(&target).is_err());
    assert!(realm.inspect_object(&handler).is_err());
}

#[test]
fn revoker_capture_and_symbol_ephemerons_follow_live_slots_and_drop_revoked_edges() {
    let mut realm = Realm::default();
    let values = realm
        .eval(
            r#"let revoke;(()=>{
        let target={},handler={},r=Proxy.revocable(target,handler);
        revoke=r.revoke;return [r.proxy,target,handler];
    })()"#,
        )
        .unwrap();
    let handles: Vec<_> = (0..3)
        .map(|index| {
            let Value::Object(handle) = realm
                .read_property(&values, &JsString::from(index.to_string().as_str()))
                .unwrap()
            else {
                panic!("Expected Proxy graph object")
            };
            handle
        })
        .collect();
    drop(values);
    realm.collect(usize::MAX).unwrap();
    assert!(
        handles
            .iter()
            .all(|handle| realm.inspect_object(handle).is_ok())
    );
    realm.eval("revoke();revoke();").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(
        handles
            .iter()
            .all(|handle| realm.inspect_object(handle).is_err())
    );
    assert_eq!(
        realm.eval("typeof revoke"),
        Ok(Value::String("function".into()))
    );
    realm.eval("revoke=undefined").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);

    let mut realm = Realm::default();
    let values = realm
        .eval(
            r#"let map=new WeakMap(),held;(()=>{
        let key=Symbol('handler'),target={n:7},handler={key};
        held=new Proxy(target,handler);let value={proxy:held};map.set(key,value);
        return [key,target,handler,value];
    })()"#,
        )
        .unwrap();
    let Value::Symbol(symbol) = realm.read_property(&values, &JsString::from("0")).unwrap() else {
        panic!("Expected handler Symbol")
    };
    let weak = symbol.downgrade();
    let Value::Object(target) = realm.read_property(&values, &JsString::from("1")).unwrap() else {
        panic!("Expected target")
    };
    let Value::Object(value) = realm.read_property(&values, &JsString::from("3")).unwrap() else {
        panic!("Expected conditional value")
    };
    drop(symbol);
    drop(values);
    realm.collect(usize::MAX).unwrap();
    assert!(weak.upgrade().is_some());
    assert!(realm.inspect_object(&target).is_ok());
    assert!(realm.inspect_object(&value).is_ok());
    assert_eq!(
        realm.read_property(&Value::Object(value.clone()), &JsString::from("proxy")),
        realm.eval("held")
    );
    assert_eq!(realm.eval("held.n"), Ok(Value::Number(7.0)));
    realm.eval("held=undefined").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(weak.upgrade().is_none());
    assert!(realm.inspect_object(&target).is_err());
    assert!(realm.inspect_object(&value).is_err());
}

#[test]
fn host_gaps_and_real_trap_argument_limits_remain_host_failures_without_key_list_limits() {
    for (name, operation, result) in [
        (
            "getOwnPropertyDescriptor",
            "Reflect.getOwnPropertyDescriptor(p,'fromAsync')",
            "undefined",
        ),
        (
            "defineProperty",
            "Reflect.defineProperty(p,'fromAsync',{value:1})",
            "true",
        ),
        ("has", "Reflect.has(p,'fromAsync')", "false"),
        ("get", "p.fromAsync", "7"),
        ("set", "Reflect.set(p,'fromAsync',7)", "true"),
        (
            "deleteProperty",
            "Reflect.deleteProperty(p,'fromAsync')",
            "true",
        ),
        ("ownKeys", "Reflect.ownKeys(p)", "[]"),
    ] {
        let mut realm = Realm::default();
        realm
            .eval(&format!(
                "let marker=0,flag=0,p=new Proxy(Array,{{{name}(){{marker++;return {result};}}}})"
            ))
            .unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try{{{operation};}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(
            realm.eval("marker===1 && flag===0"),
            Ok(Value::Boolean(true))
        );
    }
    for (name, operation) in [("apply", "p()"), ("construct", "new p()")] {
        let mut realm = Realm::new(Limits {
            max_arguments: Some(2),
            ..Limits::default()
        });
        realm.eval(&format!("let marker=0,calls=0,flag=0,p=new Proxy(function(){{}},{{get {name}(){{marker++;return function(){{calls++;}};}}}})")).unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try{{{operation};}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Limit { .. })
        ));
        assert_eq!(
            realm.eval("marker===1 && calls===0 && flag===0"),
            Ok(Value::Boolean(true))
        );
    }
    let mut realm = Realm::new(Limits {
        max_arguments: Some(2),
        ..Limits::default()
    });
    assert_eq!(realm.eval("let p=new Proxy({},{ownKeys(){return ['a','b','c','d','e','f','g','h'];}});Reflect.ownKeys(p).length"), Ok(Value::Number(8.0)));
}

#[test]
fn deep_forwarding_target_queries_descriptors_and_key_lists_use_a_standard_native_stack() {
    std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(|| {
        check(r#"
        let handler={},observedThis,observedNewTarget;
        let p=function(a,b){'use strict';observedThis=this;observedNewTarget=new.target;if(new.target){this.sum=a+b;}else{return a+b;}};
        p.x=7;for(let i=0;i<10000;i++)p=new Proxy(p,handler);
        assert(p.x===7 && 'x' in p && Reflect.set(p,'x',8));
        assert(Object.getOwnPropertyDescriptor(p,'x').value===8);
        assert(Reflect.defineProperty(p,'y',{value:9,configurable:true}));
        assert(Reflect.ownKeys(p).includes('y') && delete p.y);
        assert(Object.getPrototypeOf(p)===Function.prototype && Object.isExtensible(p));
        assert(p(2,3)===5 && observedThis===undefined && observedNewTarget===undefined);
        let receiver={};assert(Reflect.apply(p,receiver,[2,3])===5 && observedThis===receiver);
        let constructed=new p(2,3);assert(constructed.sum===5 && observedNewTarget===p);
        assert(Reflect.setPrototypeOf(p,null) && Object.getPrototypeOf(p)===null);
        assert(Reflect.preventExtensions(p) && !Reflect.isExtensible(p));
        true
        "#);
        check(r#"
        let missing={},descriptor={value:7,configurable:true},list={0:'x',length:1},length={valueOf(){return 1;}};
        for(let i=0;i<4000;i++){
            missing=new Proxy(missing,{});descriptor=new Proxy(descriptor,{});
            list=new Proxy(list,{});length=new Proxy(length,{});
        }
        let t=new Proxy(missing,{get(){return 7;}});
        assert(t.x===7);
        let d=new Proxy({},{getOwnPropertyDescriptor(){return descriptor;}});
        assert(Object.getOwnPropertyDescriptor(d,'x').value===7);
        assert(Reflect.ownKeys(new Proxy({},{ownKeys(){return list;}}))[0]==='x');
        assert(Reflect.ownKeys(new Proxy({},{ownKeys(){return {0:'x',length};}}))[0]==='x');
        true
        "#);
    }).unwrap().join().unwrap();
}

#[test]
fn recursive_user_hooks_and_opted_in_internal_cycles_abort_without_language_cleanup() {
    for (setup, operation) in [
        ("let p=new Proxy({},{get(){return p.x;}});", "p.x"),
        ("let p=new Proxy({},{get get(){return p.x;}});", "p.x"),
        (
            "let p=new Proxy({},{getOwnPropertyDescriptor(){return {configurable:true,get value(){return Object.getOwnPropertyDescriptor(p,'x').value;}};}});",
            "Object.getOwnPropertyDescriptor(p,'x')",
        ),
        (
            "let p=new Proxy({},{ownKeys(){return {length:{valueOf(){return Reflect.ownKeys(p).length;}}};}});",
            "Reflect.ownKeys(p)",
        ),
        (
            "let p=new Proxy(function(){},{apply(){return p();}});",
            "p()",
        ),
        (
            "let p=new Proxy(function(){},{construct(){return new p();}});",
            "new p()",
        ),
    ] {
        std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(move || {
                let mut realm = Realm::default();
                realm.eval(&format!("let flag=0;{setup}")).unwrap();
                assert!(matches!(
                    realm.eval(&format!(
                        "try{{{operation};}}catch{{flag=1;}}finally{{flag=2;}}"
                    )),
                    Err(Error::Limit { .. })
                ));
                assert_eq!(
                    realm.eval("flag===0 && Number('4')===4"),
                    Ok(Value::Boolean(true))
                );
            })
            .unwrap()
            .join()
            .unwrap();
    }
    for setup in [
        "let a={},p=new Proxy(a,{});Object.setPrototypeOf(a,p);",
        "let handler={},p=new Proxy({},handler);Object.setPrototypeOf(handler,p);",
    ] {
        std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(move || {
                // Only these deliberately cyclic graphs opt into a work budget.
                // No default Proxy-depth quota or enlarged native stack is added.
                let mut realm = Realm::new(Limits {
                    max_steps: Some(10000),
                    ..Limits::default()
                });
                realm.eval(&format!("let flag=0;{setup}")).unwrap();
                assert!(matches!(
                    realm.eval("try{p.missing;}catch{flag=1;}finally{flag=2;}"),
                    Err(Error::Limit { .. })
                ));
                assert_eq!(
                    realm.eval("flag===0 && Number('4')===4"),
                    Ok(Value::Boolean(true))
                );
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
