//! Synchronous ForOf acquisition, binding and closing through source awaits.

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
fn awaited_rhs_and_iteration_bindings_preserve_tdz_outer_scope_and_distinct_closures() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let x=9,result,tdz=false,restored;
        async function f(){const functions=[];for(const x of await[1,2,3]){await 0;functions.push(()=>x)}return functions.map(fn=>fn()).join(',')}
        async function g(){try{for(let x of await x){}}catch(error){tdz=error instanceof ReferenceError}restored=x}
        f().then(value=>{result=value});g();
    "#).unwrap();
    check(&mut realm, "tdz&&restored===9&&x===9&&result===undefined");
    drain(&mut realm);
    check(&mut realm, "result==='1,2,3'&&x===9");
}

#[test]
fn next_is_cached_return_is_live_and_yielded_promises_are_not_implicitly_awaited() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reads=0,steps=0,oldClose=0,closed=0,seen=[],result,promise=Promise.resolve(9);
        function next(){return{done:++steps>3,value:steps===1?promise:steps}}
        let iterator={get next(){reads++;return next},return(){oldClose++;throw 1}},source={[Symbol.iterator](){return iterator}};
        async function f(){for(const value of source){seen.push(value===promise?'promise':value);await 0;if(value===2)break}return 7}
        f().then(value=>{result=value});Object.defineProperty(iterator,'next',{value(){throw 2}});iterator.return=function(){closed++;return{}};
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "reads===1&&steps===2&&oldClose===0&&closed===1&&seen.join(',')==='promise,2'&&result===7",
    );
}

#[test]
fn step_failures_do_not_close_but_binding_failures_close_and_keep_the_original_error() {
    for stage in ["next", "done", "value", "binding"] {
        let mut realm = Realm::default();
        realm.eval(&format!("let stage='{stage}';")).unwrap();
        realm.eval(r#"
            let marker={},closeMarker={},closed=0,reason;
            let iterator={next(){if(stage==='next')throw marker;return{get done(){if(stage==='done')throw marker;return false},get value(){if(stage==='value')throw marker;return null}}},return(){closed++;throw closeMarker}},source={[Symbol.iterator](){return iterator}};
            async function f(){try{for(const {x} of source){await 0;break}}catch(error){reason=error}}
            f();
        "#).unwrap();
        drain(&mut realm);
        check(
            &mut realm,
            if stage == "binding" {
                "closed===1&&reason instanceof TypeError"
            } else {
                "closed===0&&reason===marker"
            },
        );
    }
}

#[test]
fn natural_exhaustion_and_continue_skip_return_while_nonthrow_close_errors_override() {
    for (tail, close, expected) in [
        (
            "continue",
            "closed++;throw marker",
            "closed===0&&result===7&&reason===undefined",
        ),
        (
            "break",
            "closed++;return 1",
            "closed===1&&reason instanceof TypeError",
        ),
        (
            "return 7",
            "closed++;throw marker",
            "closed===1&&reason===marker",
        ),
    ] {
        let mut realm = Realm::default();
        let source=r#"
            let closed=0,step=0,marker={},result,reason;
            let iterator={next(){return{done:++step>2,value:step}},return(){CLOSE}},input={[Symbol.iterator](){return iterator}};
            async function f(){for(let value of input){await 0;TAIL}return 7}
            f().then(value=>{result=value},value=>{reason=value});
        "#.replace("CLOSE",close).replace("TAIL",tail);
        realm.eval(&source).unwrap();
        drain(&mut realm);
        check(&mut realm, expected);
    }
}

#[test]
fn rejected_body_await_closes_once_and_preserves_the_reason_over_close_throws() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let marker={},closeMarker={},closed=0,reason;
        let iterator={next(){return{done:false,value:1}},return(){closed++;throw closeMarker}},source={[Symbol.iterator](){return iterator}};
        async function f(){try{for(const value of source){await Promise.reject(marker)}}catch(error){reason=error}}
        f();
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "reason===marker&&closed===1");
}

#[test]
fn using_bindings_dispose_after_finalizers_and_before_continue_next_or_iterator_close() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],step=0,result;
        function resource(id){return{id,[Symbol.dispose](){events.push('dispose'+this.id)}}}
        let iterator={next(){return{done:step===2,value:resource(step++)}},return(){events.push('close');return{}}},source={[Symbol.iterator](){return iterator}};
        async function f(){for(using value of source){try{await 0;if(value.id===0)continue;return 7}finally{events.push('finally'+value.id);await 0}}}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "step===2&&result===7&&events.join(',')==='finally0,dispose0,finally1,dispose1,close'",
    );
}

#[test]
fn var_assignment_and_destructuring_binding_forms_resume_without_adding_the_wrong_scope() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let outer,result;
        async function f(){const out=[];for(var x of[1,2]){await 0;out.push(x)}for(outer of[3]){await 0}let a,b;for([a,b]of[[4,5]]){await 0}for(let{n:value}of[{n:6}]){await 0;out.push(value)}return[out.join(','),x,outer,a,b]}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result.join('|')==='1,2,6|2|3|4|5'");
}

#[test]
fn iterator_and_replaced_next_function_stay_alive_across_await_then_release_after_close() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},step=0,closed=0,result;state.gate=new Promise(resolve=>{state.release=resolve});
        function cachedNext(){return{done:step++>0,value:1}}
        let iterator={next:cachedNext,return(){closed++;return{}}},source={[Symbol.iterator](){return iterator}};
        state.weakIterator=new WeakRef(iterator);state.weakNext=new WeakRef(cachedNext);
        async function f(){for(let value of source){await state.gate;break}return 7}
        f().then(value=>{result=value});iterator.next=function(){throw 1};cachedNext=null;source=null;iterator=null;
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "state.weakIterator.deref()!==undefined&&state.weakNext.deref()!==undefined&&result===undefined",
    );
    realm.eval("state.release(0)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "result===7&&closed===1&&state.weakIterator.deref()===undefined&&state.weakNext.deref()===undefined",
    );
}

#[test]
fn synchronous_close_accepts_a_promise_object_without_reading_then_or_awaiting_it() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let thenReads=0,result,closed=0,promise=Promise.resolve(9);
        Object.defineProperty(promise,'then',{get(){thenReads++;throw 1}});
        let source={[Symbol.iterator](){return{next(){return{done:false,value:1}},return(){closed++;return promise}}}};
        async function f(){for(const value of source){await 0;return 7}}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "closed===1&&thenReads===0&&result===7");
}

#[test]
fn host_abort_skips_iterator_close_finally_and_using_disposal_even_at_budget_zero() {
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},disposed=0,closed=0,finalized=0,caught=false,read;const dispose=Symbol.dispose;").unwrap();
        realm
            .eval("let resource={[dispose](){disposed++}};")
            .unwrap();
        realm
            .eval("state.weak=new WeakRef(resource[dispose]);")
            .unwrap();
        realm.eval("let source={[Symbol.iterator](){return{next(){return{done:false,value:resource}},return(){closed++;return{}}}}};").unwrap();
        let tail = if limited {
            "while(true){}"
        } else {
            "new SharedArrayBuffer(0)"
        };
        let code="async function f(){using root=resource;try{for(using value of source){read=()=>value;await 0;TAIL}}finally{finalized++}}".replace("TAIL",tail);
        realm.eval(&code).unwrap();
        realm.eval("f().catch(()=>{caught=true});").unwrap();
        realm.eval("delete resource[dispose];").unwrap();
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
            "disposed===0&&closed===0&&finalized===0&&!caught&&read()===resource&&state.weak.deref()===undefined",
        );
    }
}
