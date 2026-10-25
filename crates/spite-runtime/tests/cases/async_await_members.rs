//! Awaited member references retain base values and their reference operations.

use spite_runtime::{PromiseJobOutcome, Realm, Value};

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}

fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
}

#[test]
fn awaited_base_and_key_run_once_and_key_conversion_waits_for_fulfillment() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],result,obj={get x(){events.push('get');return this===obj?7:0}};
        state.gate=new Promise(resolve=>{state.release=resolve});
        let key={[Symbol.toPrimitive](hint){events.push(hint);return'x'}};
        async function f(){return (events.push('base'),await obj)[(events.push('key'),await state.gate)]}
        f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "events.join(',')==='base'&&result===undefined");
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='base,key'&&result===undefined",
    );
    realm.eval("state.release(key)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='base,key,string,get'&&result===7",
    );
}

#[test]
fn nullish_base_still_evaluates_the_key_but_does_not_convert_it() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let marker={},converted=0,keys=0,rejected,typeError,key={[Symbol.toPrimitive](){converted++;return'x'}};
        async function f(){return (await null)[(keys++,await Promise.reject(marker))]}
        async function g(){return (await undefined)[(keys++,await key)]}
        f().catch(value=>{rejected=value});g().catch(value=>{typeError=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "keys===2&&converted===0&&rejected===marker&&typeError instanceof TypeError",
    );
}

#[test]
fn awaited_member_calls_keep_the_receiver_and_select_the_function_before_arguments() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},result,reads=0,obj={get method(){reads++;return function(value){'use strict';return this===obj?value:0}}};
        state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){return ((await obj)[await 'method'])(await state.gate)}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "reads===1&&result===undefined");
    realm
        .eval("Object.defineProperty(obj,'method',{value(){throw 1}});state.release(9)")
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "reads===1&&result===9");
}

#[test]
fn private_fields_and_methods_keep_private_context_and_brand_errors_skip_arguments() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let field,method,reason,args=0;
        class C{#x=12;#read(value){return this.#x+value}async field(value){return(await value).#x}async call(value){return(await value).#read(await(args++,3))}}
        let obj=new C();obj.field(obj).then(value=>{field=value});obj.call(obj).then(value=>{method=value});obj.call({}).catch(value=>{reason=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "field===12&&method===15&&args===1&&reason instanceof TypeError",
    );
}

#[test]
fn delete_uses_the_reference_without_getting_the_value_and_preserves_strict_errors() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let gets=0,deleted,fixed,strictReason,obj={get x(){gets++;throw 1}};
        Object.defineProperty(obj,'fixed',{value:1,configurable:false});
        async function f(){return delete ((await obj)[await 'x'])}
        async function g(){return delete (await obj)[await 'fixed']}
        async function h(){'use strict';return delete (await obj)[await 'fixed']}
        f().then(value=>{deleted=value});g().then(value=>{fixed=value});h().catch(value=>{strictReason=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "gets===0&&deleted===true&&!Object.hasOwn(obj,'x')&&fixed===false&&strictReason instanceof TypeError",
    );
}

#[test]
fn symbol_keys_and_primitive_receivers_keep_their_identity() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let symbol=Symbol(),key=Symbol(),result,obj={[key]:7};
        Object.defineProperty(Symbol.prototype,'tag',{get(){'use strict';return this===symbol},configurable:true});
        let f=async()=>[(await obj)[await key],(await symbol)[await 'tag']];
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result[0]===7&&result[1]===true");
}

#[test]
fn base_values_stay_alive_across_awaited_keys_and_are_released_after_rejection() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},reason,marker={};state.gate=new Promise((resolve,reject)=>{state.reject=reject});
        let obj={x:7};state.weak=new WeakRef(obj);
        async function f(){return obj[await state.gate]}
        f().catch(value=>{reason=value});obj=null;
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.weak.deref()!==undefined&&reason===undefined",
    );
    realm.eval("state.reject(marker)").unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.weak.deref()===undefined&&reason===marker",
    );
}

#[test]
fn proxy_get_and_delete_traps_receive_the_completed_key_and_original_receiver() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let gets=0,deletes=0,result,deleted,proxy;
        proxy=new Proxy({x:7},{get(target,key,receiver){gets++;if(key!=='x'||receiver!==proxy)throw 1;return 7},deleteProperty(target,key){deletes++;return key==='x'}});
        async function f(){return (await 0,proxy)[await 'x']}
        async function g(){return delete proxy[await 'x']}
        f().then(value=>{result=value});g().then(value=>{deleted=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "gets===1&&deletes===1&&result===7&&deleted===true",
    );
}
