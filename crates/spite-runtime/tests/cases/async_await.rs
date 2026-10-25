//! Source Await continuation timing, contexts and cleanup.

use spite_runtime::{Error, PromiseJobOutcome, Realm, Value};

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
fn direct_await_yields_once_and_resumes_each_statement_without_replay() {
    let mut realm = Realm::default();
    realm.eval("let events=[];async function f(){events.push('a');await 0;events.push('b');await 0;events.push('c');return 3}let p=f();events.push('outside');p.then(v=>{events.push(v)})").unwrap();
    check(&mut realm, "events.join(',')==='a,outside'");
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::Completed
    ));
    check(&mut realm, "events.join(',')==='a,outside,b'");
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::Completed
    ));
    check(&mut realm, "events.join(',')==='a,outside,b,c'");
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='a,outside,b,c,3'");
}

#[test]
fn return_await_and_concise_arrows_forward_fulfillment_and_rejection() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let marker={},seen=[],id=async value=>await value;
        async function block(value){return (await value)}
        for(const f of[id,block]){
            f(Promise.resolve(7)).then(v=>{seen.push(v===7)});
            f({then(resolve){resolve(9)}}).then(v=>{seen.push(v===9)});
            f(Promise.reject(marker)).catch(e=>{seen.push(e===marker)});
        }
        let input=Promise.resolve(1),output=id(input);if(output===input)throw 1;
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "seen.length===6&&seen.every(v=>v)");
}

#[test]
fn native_await_ignores_public_then_species_and_replaced_promise_names() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let Saved=Promise,then=Promise.prototype.then,wait=Promise.resolve(7),result;
        Object.defineProperty(wait,'then',{get(){throw 1}});
        Object.defineProperty(Promise,Symbol.species,{get(){throw 2}});
        async function f(){return await wait}
        let output=f();Object.defineProperty(output,'constructor',{value:undefined});
        Reflect.apply(then,output,[v=>{result=v}]);Promise=function(){throw 3};
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

#[test]
fn unreachable_await_is_skipped_and_nested_await_contexts_stay_explicit_gaps() {
    let mut realm = Realm::default();
    realm
        .eval(
            "let result;async function f(){if(false)await missing;return 3}f().then(v=>{result=v})",
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result===3");
    realm
        .eval("let flag=0;async function g(){return {x:await (flag=1)}}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{g()}catch(e){flag=2}finally{flag=3}"),
        Err(Error::Unsupported { .. })
    ));
    check(&mut realm, "flag===0");
}

#[test]
fn resumed_functions_preserve_arguments_this_eval_and_lexical_arrow_context() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},receiver={x:7},results=[];
        state.gate=new Promise(resolve=>{state.release=resolve});
        async function ordinary(a){arguments[0]=4;await state.gate;return[this,a,arguments[0],eval('a')]}
        function maker(v){return async()=>{await state.gate;return[this,arguments[0],new.target]}}
        ordinary.call(receiver,2).then(v=>{results.push(v[0]===receiver&&v[1]===4&&v[2]===4&&v[3]===4)});
        maker.call(receiver,11)().then(v=>{results.push(v[0]===receiver&&v[1]===11&&v[2]===undefined)});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "results.length===0");
    realm.eval("state.release(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "results.length===2&&results.every(v=>v)");
}

#[test]
fn methods_private_names_super_and_dynamic_functions_resume_in_their_own_contexts() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let state={},results=[];state.gate=new Promise(resolve=>{state.release=resolve});
        class Base{get value(){return this.x}}
        class C extends Base{
            #hidden=7;
            async #read(p){await p;return this.#hidden+super.value}
            run(p){return this.#read(p)}
            static async read(p){await p;return this.x}
        }
        let c=new C;c.x=3;C.x=8;
        c.run(state.gate).then(v=>{results.push(v===10)});
        C.read(state.gate).then(v=>{results.push(v===8)});
        let AF=(async function(){}).constructor,compiled=AF('p','await p;return this.x');
        compiled.call(c,state.gate).then(v=>{results.push(v===3)});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "results.length===0");
    realm.eval("state.release(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "results.length===3&&results.every(v=>v)");
}

#[test]
fn pending_frames_keep_used_parameters_and_release_unrelated_callers_and_completed_locals() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},seen;state.gate=new Promise(resolve=>{state.release=resolve});
        async function wait(argument){await state.gate;seen=argument.x}
        (()=>{let value={x:9},unused={};state.used=new WeakRef(value);state.unused=new WeakRef(unused);state.output=wait(value)})();
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.used.deref()!==undefined&&state.unused.deref()===undefined&&seen===undefined",
    );
    realm.eval("state.release(0)").unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "seen===9&&state.used.deref()===undefined");
}

#[test]
fn using_resources_survive_suspension_and_dispose_after_return_or_rejection() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],result;state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){using resource={[Symbol.dispose](){events.push('dispose')}};events.push('start');await state.gate;events.push('end');return 7}
        f().then(v=>{result=v});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='start'&&result===undefined");
    realm.eval("state.release(0)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='start,end,dispose'&&result===7",
    );
    realm.eval(r#"
        let original={},cleanup={},suppressed;
        async function rejected(){using resource={[Symbol.dispose](){throw cleanup}};return await Promise.reject(original)}
        rejected().catch(e=>{suppressed=e});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "suppressed instanceof SuppressedError&&suppressed.error===cleanup&&suppressed.suppressed===original",
    );
}

#[test]
fn promise_resolve_getter_failure_rejects_without_suspending_and_runs_lexical_cleanup() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let marker={},events=[],reason,input=Promise.resolve(1);
        Object.defineProperty(input,'constructor',{get(){events.push('get');throw marker}});
        async function f(){using resource={[Symbol.dispose](){events.push('dispose')}};return await input}
        f().catch(e=>{reason=e});events.push('outside');
    "#).unwrap();
    check(
        &mut realm,
        "events.join(',')==='get,dispose,outside'&&reason===undefined",
    );
    drain(&mut realm);
    check(&mut realm, "reason===marker");
}

#[test]
fn resumed_host_failures_skip_rejection_and_disposal_and_release_cached_methods_even_at_budget_zero()
 {
    for limited in [false, true] {
        let mut realm = Realm::new(spite_runtime::Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        let tail = if limited {
            "while(true){}"
        } else {
            "new SharedArrayBuffer(0)"
        };
        realm.eval("let resource={[Symbol.dispose](){disposed++}},disposed=0,flag=0,read,weak=new WeakRef(resource[Symbol.dispose]);").unwrap();
        realm
            .eval(&format!(
                "async function f(){{using r=resource;read=()=>r;await 0;{tail}}}"
            ))
            .unwrap();
        realm
            .eval("let output=f();delete resource[Symbol.dispose];output.catch(()=>{flag=1});")
            .unwrap();
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        check(&mut realm, "weak.deref()!==undefined");
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
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        check(
            &mut realm,
            "disposed===0&&flag===0&&read()===resource&&weak.deref()===undefined",
        );
    }
}

#[test]
fn lexical_initializers_resume_in_list_order_and_keep_later_bindings_in_the_tdz() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],result,read;state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){read=()=>[first,second];const first=await state.gate,second=(events.push('second'),first+1),third=await second;let fourth=await third;return[first,second,third,fourth]}
        f().then(value=>{result=value});
    "#).unwrap();
    check(
        &mut realm,
        "let caught=0;try{read()}catch(e){if(e instanceof ReferenceError)caught++}caught===1&&events.length===0&&result===undefined",
    );
    realm.eval("state.release(7)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result.join(',')==='7,8,8,8'&&read().join(',')==='7,8'&&events.join(',')==='second'",
    );
}

#[test]
fn awaited_binding_patterns_initialize_defaults_and_close_iterators() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let closed=0,result,source={[Symbol.iterator](){return this},next(){return{value:5,done:false}},return(){closed++;return{}}};
        async function f(){const [first]=await source;let{value=first+1}=await{};return[first,value]}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "closed===1&&result.join(',')==='5,6'");
}

#[test]
fn awaited_function_values_keep_their_names_and_pattern_defaults_still_infer_names() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result;
        async function f(){const ordinary=()=>0,awaited=await(()=>1),{factory=()=>2}=await{};return[ordinary.name,awaited.name,factory.name]}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result.join(',')==='ordinary,,factory'");
}

#[test]
fn awaited_using_initializers_register_before_binding_and_dispose_only_initialized_resources() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],done=false;state.gate=new Promise(resolve=>{state.release=resolve});
        let first={[Symbol.dispose](){events.push('first')}},second={[Symbol.dispose](){events.push('second')}};
        async function f(){using a=first,b=await state.gate;events.push('body');return b===second}
        f().then(value=>{done=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "events.length===0&&!done");
    realm.eval("state.release(second)").unwrap();
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='body,second,first'&&done");
    realm.eval(r#"
        let marker={},reason;
        async function rejected(){using a=first,b=await Promise.reject(marker);events.push('unreachable')}
        rejected().catch(error=>{reason=error});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "reason===marker&&events.join(',')==='body,second,first,first'",
    );
}

#[test]
fn lexical_and_using_initializers_preserve_rejection_and_suppression_identity() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let original={},cleanup={},reason;
        async function f(){using resource={[Symbol.dispose](){throw cleanup}};const value=await Promise.reject(original);return value}
        f().catch(error=>{reason=error});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "reason instanceof SuppressedError&&reason.error===cleanup&&reason.suppressed===original",
    );
}

#[test]
fn var_initializers_preserve_hoisting_declarator_order_and_absent_initializer_values() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},read,result,events=[];state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){read=()=>value;var value=await (events.push('operand'),state.gate),value,other=await value;var value;return[value,other]}
        f().then(output=>{result=output});
    "#).unwrap();
    check(
        &mut realm,
        "read()===undefined&&result===undefined&&events.join(',')==='operand'",
    );
    realm.eval("state.release(7)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "read()===7&&result.join(',')==='7,7'&&events.join(',')==='operand'",
    );
}

#[test]
fn awaited_var_assignment_updates_the_original_sloppy_parameter_and_arguments_mapping() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},write,result;state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(value){write=x=>{arguments[0]=x};var value=await state.gate;return[value,arguments[0]]}
        f(2).then(output=>{result=output});write(11);
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    realm.eval("state.release(9)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result.join(',')==='9,9'");
}

#[test]
fn awaited_var_patterns_initialize_after_the_rhs_and_preserve_function_naming_rules() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,closed=0,source={[Symbol.iterator](){return this},next(){return{value:5,done:false}},return(){closed++;return{}}};
        async function f(){var[first]=await source,{value=first+1}=await{},ordinary=()=>0,anonymous=await(()=>1);return[first,value,ordinary.name,anonymous.name]}
        f().then(output=>{result=output});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "closed===1&&result.join(',')==='5,6,ordinary,'");
}

#[test]
fn awaited_var_rejection_disposes_previously_registered_resources() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let marker={},reason,disposed=0;
        async function f(){using resource={[Symbol.dispose](){disposed++}};var value=await Promise.reject(marker);return value}
        f().catch(error=>{reason=error});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "reason===marker&&disposed===1");
}
