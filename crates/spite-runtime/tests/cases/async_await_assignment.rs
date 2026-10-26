//! Assignment references and GetValue/PutValue order across Await (13.15.2).

use spite_runtime::{Error, Limits, PromiseJobOutcome, Realm, Value};

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

fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn awaited_identifier_and_nested_property_assignments_preserve_selected_references() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let release,gate=new Promise(resolve=>{release=resolve}),first,second,old={},object=old;
        async function f(){let x=1;return(x=await 7)+x}
        async function g(){return object.x=(object.y=await gate)}
        f().then(value=>{first=value});g().then(value=>{second=value});object={};
    "#,
        )
        .unwrap();
    check(
        &mut realm,
        "old.x===undefined&&old.y===undefined&&second===undefined",
    );
    realm.eval("release(9)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "first===14&&second===9&&old.x===9&&old.y===9&&object.x===undefined&&object.y===undefined",
    );
}

#[test]
fn simple_member_assignment_defers_key_coercion_until_after_the_awaited_rhs() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,gets=0,saved,receiver,object={get x(){gets++;return 1},set x(value){events.push('set');saved=value;receiver=this}};
        let key={[Symbol.toPrimitive](){events.push('coerce');return 'x'}};
        function base(){events.push('base');return object}function name(){events.push('name');return key}function rhs(){events.push('rhs');return 7}
        async function f(){return((await base())[await name()]=await rhs())}
        f().then(value=>{result=value});
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='base'&&gets===0&&saved===undefined",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "result===7&&saved===7&&receiver===object&&gets===0&&events.join(',')==='base,name,rhs,coerce,set'",
    );
}

#[test]
fn simple_nullish_assignment_runs_the_rhs_before_throwing_without_converting_its_key() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let rhs=0,coerced=0,result,key={[Symbol.toPrimitive](){coerced++;return 'x'}};
        async function f(){try{(await null)[await key]=await(rhs++,7)}catch(error){return error instanceof TypeError}}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "rhs===1&&coerced===0&&result===true");
}

#[test]
fn compound_assignment_caches_the_key_and_getter_before_rhs_but_converts_the_value_after_it() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),events=[],result,saved;
        let left={valueOf(){events.push('old-coerce');return 10}},key={[Symbol.toPrimitive](){events.push('key');return 'x'}};
        let object={get x(){events.push('get');return left},set x(value){events.push('set');saved=value}};
        async function f(){return object[key]+=await gate}
        f().then(value=>{result=value});
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='key,get'&&saved===undefined",
    );
    realm.eval("left.valueOf=function(){events.push('new-coerce');return 20};key[Symbol.toPrimitive]=()=>{throw 9};release(2)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===22&&saved===22&&events.join(',')==='key,get,new-coerce,set'",
    );
}

#[test]
fn logical_assignments_short_circuit_without_rhs_or_put_and_write_only_selected_values() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let rhs=0,sets=0,result,object={get x(){return true},set x(value){sets++}};
        async function f(){let a=0,b=1,c=7,d=null;let x=object.x||=await(rhs++,9);a&&=await(rhs++,8);b||=await(rhs++,8);c??=await(rhs++,8);d??=await(rhs++,5);a||=await(rhs++,3);return x===true&&a===3&&b===1&&c===7&&d===5}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true&&rhs===2&&sets===0");
}

#[test]
fn awaited_assignments_keep_private_brands_super_receivers_and_selected_super_bases() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),result,brand,rhs=0,events=[];
        class Base{get x(){events.push('get');return 3}set x(value){events.push('set');this.saved=value}}
        class C extends Base{#x=1;async f(){(await this).#x=await 7;super.x+=await gate;return this.#x+this.saved}async bad(){try{(await {}).#x=await(rhs++,9)}catch(error){return error instanceof TypeError}}}
        let object=new C();object.f().then(value=>{result=value});object.bad().then(value=>{brand=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='get'&&result===undefined&&brand===true&&rhs===1",
    );
    realm
        .eval("Object.setPrototypeOf(C.prototype,{set x(value){throw 9}});release(2)")
        .unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===12&&object.saved===5&&events.join(',')==='get,set'",
    );
}

#[test]
fn saved_object_environment_bindings_ignore_later_unscopables_changes() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),state={},result,x=5,holder={x:1,[Symbol.unscopables]:{x:false}};
        with(holder){state.f=async()=>{x=await gate;return x}}
        state.f().then(value=>{result=value});holder[Symbol.unscopables].x=true;
    "#).unwrap();
    realm.eval("release(7)").unwrap();
    drain(&mut realm);
    check(&mut realm, "holder.x===7&&x===5&&result===5");
}

#[test]
fn awaited_writes_reuse_strict_sloppy_immutable_and_rejection_rules() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let results=[],rhs=0,writes=0,reason={},object={set x(value){writes++}};
        async function strict(){'use strict';try{missingStrict=await(rhs++,7)}catch(error){return error instanceof ReferenceError}}
        async function sloppy(){missingSloppy=await 9;return missingSloppy===9}
        async function constant(){const x=1;try{x=await 2}catch(error){return error instanceof TypeError&&x===1}}
        async function rejected(){try{object.x=await Promise.reject(reason)}catch(error){return error===reason}}
        strict().then(value=>{results.push(value)});sloppy().then(value=>{results.push(value)});constant().then(value=>{results.push(value)});rejected().then(value=>{results.push(value)});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===4&&results.every(value=>value)&&rhs===1&&writes===0&&this.missingStrict===undefined&&this.missingSloppy===9",
    );
}

#[test]
fn pending_assignment_references_trace_raw_bases_keys_and_compound_values_then_release_them() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,releaseSecond,gate=new Promise(resolve=>{release=resolve}),second=new Promise(resolve=>{releaseSecond=resolve}),baseWeak,keyWeak,leftWeak,compoundKeyWeak,first,other;
        function base(){const object={};baseWeak=new WeakRef(object);return object}
        function key(){const object={[Symbol.toPrimitive](){return 'x'}};keyWeak=new WeakRef(object);return object}
        function compoundKey(){const object={[Symbol.toPrimitive](){return 'x'}};compoundKeyWeak=new WeakRef(object);return object}
        function compound(){return{get x(){const object={valueOf(){return 3}};leftWeak=new WeakRef(object);return object},set x(value){}}}
        async function a(){base()[key()]=await gate;0;await second;return 7}
        async function b(){compound()[compoundKey()]+=await gate;0;await second;return 8}
        a().then(value=>{first=value});b().then(value=>{other=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "baseWeak.deref()!==undefined&&keyWeak.deref()!==undefined&&leftWeak.deref()!==undefined&&compoundKeyWeak.deref()===undefined&&first===undefined&&other===undefined",
    );
    realm.eval("release(2)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "baseWeak.deref()===undefined&&keyWeak.deref()===undefined&&leftWeak.deref()===undefined&&first===undefined&&other===undefined",
    );
    realm.eval("releaseSecond(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "first===7&&other===8");
}

#[test]
fn assignments_in_for_updates_and_named_rhs_evaluation_keep_existing_semantics() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,events=[],object={set x(value){events.push('put');this.value=value}};
        async function f(){let total=0;for(let i=await 0;await(i<3);i=await(i+1)){total+=i;await 0}(await object).x=class{static{events.push(this.name)}};return total}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===3&&events.join(',')===',put'&&object.value.name===''",
    );
}

#[test]
fn abandoned_assignment_references_skip_put_handlers_and_disposal_on_host_abort() {
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},writes=0,disposed=0,finalized=0,caught=false;const dispose=Symbol.dispose;").unwrap();
        realm
            .eval("let object={set x(value){writes++}},resource={[dispose](){disposed++}};")
            .unwrap();
        realm
            .eval("state.weak=new WeakRef(resource[dispose]);")
            .unwrap();
        realm.eval("function spin(){while(true){}}").unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "new SharedArrayBuffer(0)"
        };
        let code="async function f(){using root=resource;try{object.x=(await 0,TAIL)}finally{finalized++}}".replace("TAIL",tail);
        realm.eval(&code).unwrap();
        realm.eval("f().catch(()=>{caught=true});").unwrap();
        realm.eval("delete resource[dispose]").unwrap();
        let mut reported = false;
        loop {
            match realm.run_next_promise_job().unwrap() {
                PromiseJobOutcome::NoJob => break,
                PromiseJobOutcome::Completed => (),
                PromiseJobOutcome::Reported(error) => {
                    assert!(matches!(error, Error::Limit { .. }) == limited, "{error}");
                    if !limited {
                        assert!(matches!(error, Error::Unsupported { .. }), "{error}")
                    }
                    reported = true;
                }
            }
        }
        assert!(reported);
        collect(&mut realm);
        check(
            &mut realm,
            "writes===0&&disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}
