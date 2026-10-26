//! Suspended object BindingInitialization, keyed bindings and rest (14.3.3).

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
fn keys_are_converted_once_and_defaults_finish_before_later_properties_and_declarators() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,key={[Symbol.toPrimitive](hint){events.push(hint);return 'x'}},source={get x(){events.push('get');return undefined},get y(){events.push('y');return 9}};
        async function f(){let {[await(events.push('key'),key)]:x=await(events.push('default'),7),y}=await source,tail=(events.push('tail'),11);return [x,y,tail].join(',')}f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "events.length===0&&result===undefined");
    drain(&mut realm);
    check(
        &mut realm,
        "result==='7,9,11'&&events.join(',')==='key,string,get,default,y,tail'",
    );
}

#[test]
fn var_targets_resolve_after_keys_but_before_getters_and_survive_suspension() {
    for awaited_key in [false, true] {
        let mut realm = Realm::default();
        realm.eval("let blocked={x:false},holder={x:1,[Symbol.unscopables]:blocked},source={get a(){blocked.x=true;return undefined}},result;").unwrap();
        let key = if awaited_key {
            "await(blocked.x=true,'a')"
        } else {
            "'a'"
        };
        realm.eval(&format!("async function f(x){{with(holder){{var {{[{key}]:x=await 7}}=source}}return [x,arguments[0]].join(',')}}f(3).then(value=>{{result=value}})")).unwrap();
        drain(&mut realm);
        check(
            &mut realm,
            if awaited_key {
                "result==='7,7'&&holder.x===1"
            } else {
                "result==='3,3'&&holder.x===7"
            },
        );
    }
}

#[test]
fn primitive_getv_receivers_and_rest_data_properties_use_shared_native_operations() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let seen,result;Object.defineProperty(String.prototype,'missing',{get(){'use strict';seen=typeof this;return undefined},configurable:true});
        async function f(){let {missing=await 7,...rest}='ab';return missing===7&&seen==='string'&&rest[0]==='a'&&rest[1]==='b'&&Object.getPrototypeOf(rest)===Object.prototype&&Object.getOwnPropertyDescriptor(rest,'0').writable}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true");
}

#[test]
fn rest_excludes_repeated_symbol_keys_and_reads_current_own_descriptors_after_defaults() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let key=Symbol(),release,gate=new Promise(resolve=>{release=resolve}),reads=0,result,source={[key]:undefined,a:1};
        Object.defineProperty(source,'hidden',{value:2,enumerable:false});Object.defineProperty(source,'live',{enumerable:true,get(){reads++;return 11}});
        async function f(){let {[await key]:x=await gate,[key]:y=9,...rest}=source;return x===7&&y===9&&!Object.hasOwn(rest,key)&&rest.a===3&&rest.added===13&&!Object.hasOwn(rest,'hidden')&&rest.live===11}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "reads===0&&result===undefined");
    realm.eval("source.a=3;source.added=13;release(7)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true&&reads===1");
}

#[test]
fn mixed_nested_patterns_close_array_iterators_inside_out_on_default_rejection() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},events=[],result;function source(name,value){return{[Symbol.iterator](){return this},next(){return{done:false,value}},return(){events.push(name);throw 0}}}
        async function f(){try{let [{a:[{b:x=await Promise.reject(reason)}]}]=source('outer',{a:source('inner',{})})}catch(error){return error===reason}}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===true&&events.join(',')==='inner,outer'",
    );
}

#[test]
fn catch_objects_wait_for_parameter_initialization_and_unwind_through_finally() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},events=[],result;async function f(){try{try{throw{a:[]}}catch({[await 'a']:[x=await Promise.reject(reason)]}){events.push('wrong')}finally{await 0;events.push('finally')}}catch(error){events.push('outer');return error===reason}}
        async function g(){try{throw{}}catch({x=await 7}){return()=>x}}let get;f().then(value=>{result=value});g().then(value=>{get=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===true&&get()===7&&events.join(',')==='finally,outer'",
    );
}

#[test]
fn iteration_heads_and_three_clause_headers_keep_fresh_cells_and_parameter_aliases() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let results=[],closures=[];async function a(x){for(var {[await 'x']:x=await 7} of [{}]){}return arguments[0]}
        async function b(){for await(let {x=await 9} of [{},{}]){closures.push(()=>x);x++}return closures.map(f=>f()).join(',')}
        async function c(){let saved=[];for(let {[await 'x']:x=await 11}={};x<13;x++){saved.push(()=>x)}return saved.map(f=>f()).join(',')}
        async function d(){for(const {length=await missing} in {abc:1})return length}
        a(1).then(value=>results.push(value));b().then(value=>results.push(value));c().then(value=>results.push(value));d().then(value=>results.push(value));
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===4&&results.includes(7)&&results.includes('10,10')&&results.includes('11,12')&&results.includes(3)",
    );
}

#[test]
fn coercibility_and_tdz_errors_skip_unreachable_key_operands_and_outer_iterator_closing_is_ordered()
{
    let mut realm = Realm::default();
    realm.eval(r#"
        let effects=0,results=[],events=[],reason={};async function a(){try{let {[await(effects++,'x')]:x}=null}catch(error){return error instanceof TypeError}}
        async function b(){try{let {x=await y,y=7}={}}catch(error){return error instanceof ReferenceError}}
        let source={[Symbol.iterator](){return this},next(){return{done:false,value:{get x(){throw reason}}}},return(){events.push('close');return{}}};
        async function c(){try{let [{x=await missing}]=source}catch(error){return error===reason}}
        a().then(value=>results.push(value));b().then(value=>results.push(value));c().then(value=>results.push(value));
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "effects===0&&results.length===3&&results.every(x=>x===true)&&events.join(',')==='close'",
    );
}

#[test]
fn source_objects_remain_live_while_computed_keys_wait_and_are_released_after_binding() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),weak,result;function source(){let value={x:7};weak=new WeakRef(value);return value}
        async function f(){let {[await gate]:x}=source();return x}f().then(value=>{result=value});
    "#).unwrap();
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined&&result===undefined");
    realm.eval("release('x')").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(&mut realm, "result===7&&weak.deref()===undefined");
}

#[test]
fn defined_values_and_ordinary_defaults_keep_skipping_and_name_inference() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let effects=0,result;async function f(){let {x=await(effects++,missing),fn=function(){},klass=class{},anon=await(()=>{})}={x:7};for(const {[await(effects++,'x')]:x}of []){}return x===7&&fn.name==='fn'&&klass.name==='klass'&&anon.name===''&&effects===0}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true&&effects===0");
}

#[test]
fn host_failure_in_keys_or_defaults_skips_parent_close_disposal_and_handlers() {
    for limited in [false, true] {
        for key in [false, true] {
            let mut realm = Realm::new(Limits {
                max_steps: limited.then_some(10_000),
                ..Default::default()
            });
            realm.eval("let closed=0,disposed=0,finalized=0,caught=false,proceed,gate=new Promise(resolve=>{proceed=resolve});").unwrap();
            realm.eval("function spin(){while(true){}}let resource={[Symbol.dispose](){disposed++}},weak=new WeakRef(resource[Symbol.dispose]),source={[Symbol.iterator](){return this},next(){return{done:false,value:{}}},return(){closed++;return{}}};").unwrap();
            let tail = if limited {
                "spin()"
            } else {
                "new SharedArrayBuffer(0)"
            };
            let pattern = if key {
                format!("{{[(await gate,{tail})]:x}}")
            } else {
                format!("{{x=(await gate,{tail})}}")
            };
            realm.eval(&format!("async function f(){{using r=resource;try{{let [{pattern}]=source}}catch{{caught=true}}finally{{finalized++}}}}f().catch(()=>{{caught=true}})")).unwrap();
            realm
                .eval("delete resource[Symbol.dispose];proceed()")
                .unwrap();
            let mut reported = false;
            loop {
                match realm.run_next_promise_job().unwrap() {
                    PromiseJobOutcome::NoJob => break,
                    PromiseJobOutcome::Completed => (),
                    PromiseJobOutcome::Reported(error) => {
                        assert!(matches!(error, Error::Limit { .. }) == limited, "{error}");
                        if !limited {
                            assert!(matches!(error, Error::Unsupported { .. }), "{error}");
                        }
                        reported = true;
                    }
                }
            }
            assert!(reported);
            collect(&mut realm);
            check(
                &mut realm,
                "closed===0&&disposed===0&&finalized===0&&!caught&&weak.deref()===undefined",
            );
        }
    }
}

#[test]
fn private_context_dynamic_functions_and_previous_object_boundary_programs_now_execute() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let results=[],operand=0,handled=0;class C{#x=7;async f(){let {x=await this.#x}={};return x}}
        new C().f().then(value=>results.push(value));let AF=(async function(){}).constructor;AF('let {[await "x"]:x=await 13}={};return x')().then(value=>results.push(value));
        async function a(){var {x=await(operand++)}={};return x}async function b(){let {x=await(operand++)}={};return x}async function c(){for(var {x=await(operand++)}of [{}]){handled++}return x}
        a().then(value=>results.push(value));b().then(value=>results.push(value));c().then(value=>results.push(value));
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===5&&results.includes(7)&&results.includes(13)&&results.includes(0)&&results.includes(1)&&results.includes(2)&&operand===3&&handled===1",
    );
}
