//! Lexical resource lifetimes, abrupt completions and iterator cleanup.

use crate::common::UnavailableHostRealm;
use spite_runtime::{Error, Limits, PromiseJobOutcome, Realm, Value};

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}

fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => {}
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
}

#[test]
fn block_bindings_cache_disposers_and_preserve_receivers_and_reverse_order() {
    check(
        &mut Realm::default(),
        r#"
        let log=[],gets=0,read;
        let first={
            get[Symbol.dispose](){
                gets++;
                return function(){'use strict';if(this!==first||arguments.length!==0)throw 1;log.push('first');return{get then(){throw 2}}};
            }
        };
        {using a=first,b={[Symbol.dispose](){if(read()!==first)throw 3;log.push('second')}};
            read=()=>a;Object.defineProperty(first,Symbol.dispose,{value(){throw 4}});
            let caught=false;try{a=null}catch(e){caught=e instanceof TypeError}if(!caught)throw 5;
        }
        gets===1&&read()===first&&log.join(',')==='second,first'
    "#,
    );
}

#[test]
fn resource_cleanup_preserves_return_values_break_continue_and_normal_values() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let log=[];function f(){using a={[Symbol.dispose](){log.push('return');return 99}};return 7}
        if(f()!==7)throw 1;
        for(let i=0;i<3;i++){using r={[Symbol.dispose](){log.push(i)}};if(i===0)continue;if(i===1)break;}
        outer:{using r={[Symbol.dispose](){log.push('label')}};break outer;}
        log.join(',')==='return,0,1,label'
    "#,
    );
    assert_eq!(
        realm.eval("1;{using r={[Symbol.dispose](){return 99}};}"),
        Ok(Value::Number(1.0))
    );
}

#[test]
fn body_and_cleanup_errors_form_intrinsic_suppression_chains() {
    check(
        &mut Realm::default(),
        r#"
        let original={},a={},b={},caught,Saved=SuppressedError;
        SuppressedError=function(){throw 9};
        try{{using first={[Symbol.dispose](){throw a}},second={[Symbol.dispose](){throw b}};throw original}}catch(e){caught=e}
        Object.getPrototypeOf(caught)===Saved.prototype&&caught.error===a&&caught.suppressed.error===b&&caught.suppressed.suppressed===original&&Error.isError(caught)&&Object.getOwnPropertyNames(caught).join(',')==='error,suppressed'
    "#,
    );
}

#[test]
fn initializer_and_lookup_failures_dispose_prior_resources_before_binding_initialization() {
    check(
        &mut Realm::default(),
        r#"
        let log=[],read,caught;
        try{{using first={[Symbol.dispose](){log.push('disposed')}},x={get[Symbol.dispose](){read=()=>x;return x}};}}catch(e){caught=e}
        let tdz=false;try{read()}catch(e){tdz=e instanceof ReferenceError}
        caught instanceof ReferenceError&&tdz&&log.join(',')==='disposed'
    "#,
    );
}

#[test]
fn three_clause_headers_keep_resources_until_the_whole_loop_exits() {
    check(
        &mut Realm::default(),
        r#"
        let log=[],i=0,read;
        for(using r={[Symbol.dispose](){log.push('dispose '+i)}};i<2;i++){read=()=>r;log.push('body '+i)}
        let marker={},failed=[];try{for(using a={[Symbol.dispose](){failed.push(1)}},b=(()=>{throw marker})();false;){}}catch(e){if(e!==marker)throw e}
        typeof read()==='object'&&log.join(',')==='body 0,body 1,dispose 2'&&failed.join(',')==='1'
    "#,
    );
    check(
        &mut Realm::default(),
        r#"
        let log=[],j=0,resource=tag=>({[Symbol.dispose](){log.push(tag)}});
        outer:for(using top=resource('top');j<2;j++){
            using body=resource('body '+j);
            for(using item of[resource('inner '+j)]){log.push('run '+j);continue outer;}
        }
        log.join(',')==='run 0,inner 0,body 0,run 1,inner 1,body 1,top'
    "#,
    );
}

#[test]
fn for_of_resources_dispose_per_iteration_before_next_and_iterator_close() {
    check(
        &mut Realm::default(),
        r#"
        let log=[],i=0,reads=[],iterable={
            [Symbol.iterator](){
                return{
                    next(){log.push('next');let n=i++;return{done:n===3,value:{n,[Symbol.dispose](){log.push('dispose '+this.n)}}}},
                    return(){log.push('close');return{}}
                };
            }
        };
        for(using r of iterable){reads.push(()=>r.n);log.push('body '+r.n);if(r.n===0)continue;break;}
        reads[0]()===0&&reads[1]()===1&&log.join(',')==='next,body 0,dispose 0,next,body 1,dispose 1,close'
    "#,
    );
}

#[test]
fn for_of_rhs_has_a_tdz_and_disposal_failures_close_without_replacing_throws() {
    check(
        &mut Realm::default(),
        r#"
        let x=[],tdz=false;try{for(using x of x){}}catch(e){tdz=e instanceof ReferenceError}
        let body={},cleanup={},close={},caught,closed=0;
        let iterable={[Symbol.iterator](){return{next(){return{done:false,value:{[Symbol.dispose](){throw cleanup}}}},return(){closed++;throw close}}}};
        try{for(using r of iterable){throw body}}catch(e){caught=e}
        tdz&&closed===1&&caught instanceof SuppressedError&&caught.error===cleanup&&caught.suppressed===body
    "#,
    );
}

#[test]
fn functions_methods_static_blocks_eval_and_async_returns_close_their_own_scopes() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let log=[],done=false;function r(tag){return{[Symbol.dispose](){log.push(tag)}}}
        function f(){using value=r('function');return 7}if(f()!==7)throw 1;
        let a=()=>{using value=r('arrow')};a();
        class C{static{using value=r('static')}m(){using value=r('method')}}new C().m();
        eval("{using value=r('eval');}");
        async function g(){using value=r('async');return 9}g().then(v=>{done=v===9});
    "#,
        )
        .unwrap();
    check(
        &mut realm,
        "!done&&log.join(',')==='function,arrow,static,method,eval,async'",
    );
    drain(&mut realm);
    check(&mut realm, "done");
}

#[test]
fn nullish_inputs_primitives_and_named_function_initializers_follow_using_rules() {
    check(
        &mut Realm::default(),
        r#"
        let caught=0,log=[];
        {using a=null,b=undefined;}
        for(const value of[0,false,'',1n,Symbol()]){try{{using r=value;}}catch(e){if(e instanceof TypeError)caught++}}
        Function.prototype[Symbol.dispose]=function(){log.push(this.name)};
        {using arrow=()=>{},fn=function(){},cls=class{};}
        delete Function.prototype[Symbol.dispose];
        caught===5&&log.join(',')==='cls,fn,arrow'
    "#,
    );
}

#[test]
fn finalizers_follow_inner_disposal_and_stacks_share_the_disposal_protocol() {
    check(
        &mut Realm::default(),
        r#"
        let log=[];function f(){try{using s=new DisposableStack;s.defer(()=>{log.push('stack')});{using r={[Symbol.dispose](){log.push('inner')}};return 3}}finally{using r={[Symbol.dispose](){log.push('final resource')}};log.push('finally')}}
        f()===3&&log.join(',')==='inner,stack,finally,final resource'
    "#,
    );
}

#[test]
fn host_aborts_skip_disposal_but_release_cached_methods_from_escaped_environments() {
    let mut realm = Realm::with_unavailable_host();
    realm.eval("let state={value:{[Symbol.dispose](){throw 1}}},flag=0;state.weak=new WeakRef(state.value[Symbol.dispose])").unwrap();
    assert!(matches!(realm.eval("try{{using r=state.value;state.read=()=>r;r[Symbol.dispose]=null;spiteUnavailableHost()}}catch(e){flag=1}finally{flag=2}"),Err(Error::Unsupported{..})));
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "flag===0&&state.read()===state.value&&state.weak.deref()===undefined",
    );
}

#[test]
fn opted_in_work_abort_discards_prepaid_captures_without_running_cleanup() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm
        .eval("let state={value:{[globalThis.Symbol.dispose](){throw 1}}}")
        .unwrap();
    realm
        .eval("state.weak=new globalThis.WeakRef(state.value[globalThis.Symbol.dispose])")
        .unwrap();
    assert!(matches!(
        realm.eval("{using r=state.value;state.read=()=>r;r[globalThis.Symbol.dispose]=null;state.started=true;while(true){}}"),
        Err(Error::Limit { .. })
    ));
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.started&&state.read()===state.value&&state.weak.deref()===undefined",
    );
}
