//! Suspended lexical array bindings in ForIn/Of heads (14.7.5, 14.3.3).

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
fn for_of_defaults_use_fresh_mutable_cells_and_skip_defined_values() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let effects=0,closures=[],result;async function f(){for(let [x=await(effects++,7)] of [[],[9],[]]){closures.push(()=>x);x++;await 0}return closures.map(f=>f()).join(',')}f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "effects===1&&result===undefined");
    drain(&mut realm);
    check(
        &mut realm,
        "effects===2&&result==='8,10,8'&&typeof x==='undefined'",
    );
}

#[test]
fn for_await_waits_for_binding_defaults_before_advancing_the_outer_iterator() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),calls=0,values=[],result;
        let source={[Symbol.asyncIterator](){return this},next(){return ++calls===1?{done:false,value:[]}:calls===2?{done:false,value:[9]}:{done:true}}};
        async function f(){for await(const [x=await gate] of source){values.push(x)}return values.join(',')}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "calls===1&&values.length===0&&result===undefined",
    );
    realm.eval("release(7)").unwrap();
    drain(&mut realm);
    check(&mut realm, "calls===3&&result==='7,9'");
}

#[test]
fn for_in_destructures_selected_string_keys_with_awaited_defaults() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let effects=0,values=[],result;async function f(){for(const [x=await(effects++,7)] in {'':1,a:2}){values.push(x);await 0}return values.join(',')}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "effects===1&&result==='7,a'&&typeof x==='undefined'",
    );
}

#[test]
fn binding_close_precedes_body_finalizers_and_labelled_outer_close() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,calls=0;
        function inner(){return{[Symbol.iterator](){return this},next(){return{done:false,value:undefined}},return(){events.push('inner');return{done:true}}}}
        let source={[Symbol.asyncIterator](){return this},next(){calls++;return{done:false,value:inner()}},return(){events.push('outer');return Promise.resolve({done:true})}};
        async function f(){outer:for await(const [x=await 7] of source){try{events.push('body'+calls);if(calls===1)continue outer;break outer}finally{await 0;events.push('finally')}}return events.join(',')}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "calls===2&&result==='inner,body1,finally,inner,body2,finally,outer'",
    );
}

#[test]
fn default_rejection_closes_binding_iterators_before_sync_or_async_outer_iterators() {
    for asynchronous in [false, true] {
        let mut realm = Realm::default();
        realm.eval("let reason={},events=[],result;let inner={[Symbol.iterator](){return this},next(){return{done:false,value:undefined}},return(){events.push('inner');return{done:true}}};let outer={[Symbol.iterator](){return this},[Symbol.asyncIterator](){return this},next(){return{done:false,value:inner}},return(){events.push('outer');return{done:true}}}").unwrap();
        let keyword = if asynchronous { "for await" } else { "for" };
        realm.eval(&format!("async function f(){{try{{{keyword}(const [x=await Promise.reject(reason)] of outer){{}}}}catch(error){{await 0;return error===reason}}}}f().then(value=>{{result=value}})")).unwrap();
        drain(&mut realm);
        check(
            &mut realm,
            "result===true&&events.join(',')==='inner,outer'&&typeof x==='undefined'",
        );
    }
}

#[test]
fn const_defaults_preserve_names_immutability_method_and_dynamic_contexts() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let names=[],readonly=false,result,dynamic;class C{async f(){for(const [callback=function(){},x=await 7] of [[]]){names.push(callback.name);try{x=2}catch(error){readonly=error instanceof TypeError}return x}}}new C().f().then(value=>{result=value});
        (async function(){}).constructor('for(const [x=await 13] of [[]])return x')().then(value=>{dynamic=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===7&&dynamic===13&&names.join(',')==='callback'&&readonly",
    );
}

#[test]
fn unimplemented_iteration_pattern_forms_remain_host_boundaries() {
    for header in [
        "for(var [x=await(operand++)] of [[]])",
        "for(const {x=await(operand++)} of [{}])",
        "for([x=await(operand++)] of [[]])",
    ] {
        let mut realm = Realm::default();
        realm.eval("let before=0,operand=0,handled=0").unwrap();
        assert!(matches!(realm.eval(&format!("async function f(){{var x;before++;try{{{header}{{}}}}catch{{handled=1}}finally{{handled=2}}}}f().catch(()=>{{handled=3}})")),Err(Error::Unsupported{..})),"{header}");
        check(&mut realm, "before===1&&operand===0&&handled===0");
    }
}

#[test]
fn host_abort_in_a_head_default_skips_inner_and_outer_close_and_root_cleanup() {
    for kind in ["of", "in", "await"] {
        for limited in [false, true] {
            let mut realm = Realm::new(Limits {
                max_steps: limited.then_some(10_000),
                ..Default::default()
            });
            realm.eval("let state={},innerClosed=0,outerClosed=0,disposed=0,finalized=0,caught=false,proceed,gate=new Promise(resolve=>{proceed=resolve});const dispose=Symbol.dispose;").unwrap();
            realm.eval("let resource={[dispose](){disposed++}};state.weak=new WeakRef(resource[dispose]);function spin(){while(true){}};let inner={[Symbol.iterator](){return this},next(){return{done:false,value:undefined}},return(){innerClosed++;return{done:true}}};let outer={[Symbol.iterator](){return this},[Symbol.asyncIterator](){return this},next(){return{done:false,value:inner}},return(){outerClosed++;return{done:true}}}").unwrap();
            let tail = if limited {
                "spin()"
            } else {
                "new SharedArrayBuffer(0)"
            };
            let header = match kind {
                "of" => format!("for(const [x=(await gate,{tail})] of outer)"),
                "in" => format!("for(const [x=(await gate,{tail})] in {{'':1}})"),
                _ => format!("for await(const [x=(await gate,{tail})] of outer)"),
            };
            realm.eval(&format!("async function f(){{using r=resource;try{{{header}{{}}}}catch{{caught=true}}finally{{finalized++}}}}f().catch(()=>{{caught=true}})")).unwrap();
            drain(&mut realm);
            realm.eval("delete resource[dispose];proceed()").unwrap();
            let mut reported = false;
            loop {
                match realm.run_next_promise_job().unwrap() {
                    PromiseJobOutcome::NoJob => break,
                    PromiseJobOutcome::Completed => (),
                    PromiseJobOutcome::Reported(error) => {
                        assert!(
                            matches!(error, Error::Limit { .. }) == limited,
                            "{kind}: {error}"
                        );
                        if !limited {
                            assert!(matches!(error, Error::Unsupported { .. }), "{error}")
                        }
                        reported = true;
                    }
                }
            }
            assert!(reported, "{kind}");
            collect(&mut realm);
            check(
                &mut realm,
                "innerClosed===0&&outerClosed===0&&disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
            );
        }
    }
}
