//! Suspended var Array BindingInitialization and pre-step References (14.3.2).

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
fn var_defaults_preserve_parameter_aliases_multiple_declarators_and_for_headers() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,header;async function f(x){var [x=await 7]=await [],[y=await 9]=[],tail=11;return [x,arguments[0],y,tail].join(',')}
        async function g(x){for(var [x=await 7]=[];x<9;x++){}return [x,arguments[0]].join(',')}f(1).then(value=>{result=value});g(1).then(value=>{header=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result==='7,7,9,11'&&header==='9,9'");
}

#[test]
fn with_references_resolve_before_stepping_and_keep_the_selected_object_target() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],blocked={x:false},holder={x:1,get [Symbol.unscopables](){events.push('resolve');return blocked}},result;
        let source={[Symbol.iterator](){return this},get next(){events.push('cache');return function(){events.push('next');blocked.x=true;return{done:false,value:undefined}}},return(){events.push('close');return{done:true}}};
        async function f(){with(holder){var [x=await(events.push('default'),7)]=source}return x}f().then(value=>{result=value});
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='cache,resolve,next,default'&&holder.x===1",
    );
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='cache,resolve,next,default,close'&&holder.x===7&&result===undefined",
    );
}

#[test]
fn cached_object_references_survive_await_and_pattern_references_resolve_after_the_rhs() {
    for rhs_await in [false, true] {
        let mut realm = Realm::default();
        realm.eval("let release,gate=new Promise(resolve=>{release=resolve}),blocked={x:false},holder={x:1,[Symbol.unscopables]:blocked},result").unwrap();
        if rhs_await {
            realm.eval("blocked.x=true").unwrap();
        }
        let rhs = if rhs_await {
            "await(blocked.x=false,[])"
        } else {
            "[]"
        };
        realm.eval(&format!("async function f(){{with(holder){{var [x=await gate]={rhs}}}return x}}f().then(value=>{{result=value}})")).unwrap();
        drain(&mut realm);
        realm
            .eval("blocked.x=true;delete holder.x;release(7)")
            .unwrap();
        drain(&mut realm);
        check(&mut realm, "holder.x===7&&result===undefined");
    }
}

#[test]
fn references_still_resolve_for_each_binding_after_iterator_exhaustion() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],holder={x:0,y:0,get [Symbol.unscopables](){events.push('resolve');return {}}},result;
        let source={[Symbol.iterator](){return this},next(){events.push('next');return{done:true}},return(){events.push('wrong');return{done:true}}};
        async function f(){with(holder){var [x=await(events.push('dx'),7),y=await(events.push('dy'),9)]=source}return x===undefined&&y===undefined}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='resolve,next,dx,resolve,dy'&&holder.x===7&&holder.y===9&&result===true",
    );
}

#[test]
fn rest_references_are_resolved_before_collecting_remaining_iterator_values() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let blocked={first:false,rest:false},holder={first:0,rest:0,[Symbol.unscopables]:blocked},result,calls=0;
        let source={[Symbol.iterator](){return this},next(){calls++;if(calls===2)blocked.rest=true;return calls===1?{done:false,value:undefined}:calls===2?{done:false,value:11}:calls===3?{done:false,value:12}:{done:true}}};
        async function f(){with(holder){var [first=await 7,...rest]=source}return first===undefined&&rest===undefined}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===true&&holder.first===7&&holder.rest.join(',')==='11,12'&&calls===4",
    );
}

#[test]
fn var_iteration_defaults_update_mapped_parameters_across_all_three_loop_kinds() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let results=[];async function a(x){for(var [x=await 7] of [[]]){}return arguments[0]}
        async function b(x){for await(var [x=await 9] of [[]]){}return arguments[0]}
        async function c(x){for(var [x=await 11] in {'':1}){}return arguments[0]}
        a(1).then(value=>results.push(value));b(1).then(value=>results.push(value));c(1).then(value=>results.push(value));
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===3&&results.includes(7)&&results.includes(9)&&results.includes(11)",
    );
}

#[test]
fn nested_var_defaults_keep_names_and_previously_unsupported_programs_now_execute() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,names,before=0,operand=0,handled=0;async function f(){before++;try{var [x=await(operand++)]=[]}catch{handled=1}finally{handled=2}return x}
        async function g(x){var [[x=await 7],fn=function(){}]=await [[],undefined];return x===7&&arguments[0]===7&&fn.name==='fn'}f().then(value=>{result=value});g(1).then(value=>{names=value});
    "#).unwrap();
    check(&mut realm, "before===1&&operand===1&&handled===0");
    drain(&mut realm);
    check(&mut realm, "result===0&&names===true&&handled===2");
}

#[test]
fn default_rejection_closes_each_nested_iterator_before_entering_outer_handlers() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},events=[],result;function source(name,value){return{[Symbol.iterator](){return this},next(){return{done:false,value}},return(){events.push(name);return{done:true}}}}
        async function f(){try{var [[x=await Promise.reject(reason)]]=source('outer',source('inner',undefined))}catch(error){await 0;return error===reason&&x===undefined}}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===true&&events.join(',')==='inner,outer'",
    );
}

#[test]
fn host_abort_in_var_defaults_skips_iterator_close_disposal_and_handlers() {
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},closed=0,disposed=0,finalized=0,caught=false,proceed,gate=new Promise(resolve=>{proceed=resolve});const dispose=Symbol.dispose;").unwrap();
        realm.eval("let resource={[dispose](){disposed++}};state.weak=new WeakRef(resource[dispose]);function spin(){while(true){}};let source={[Symbol.iterator](){return this},next(){return{done:false,value:undefined}},return(){closed++;return{done:true}}}").unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "new SharedArrayBuffer(0)"
        };
        realm.eval(&format!("async function f(){{using r=resource;try{{var [x=(await gate,{tail})]=source}}catch{{caught=true}}finally{{finalized++}}}}f().catch(()=>{{caught=true}})")).unwrap();
        realm.eval("delete resource[dispose];proceed()").unwrap();
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
            "closed===0&&disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}
