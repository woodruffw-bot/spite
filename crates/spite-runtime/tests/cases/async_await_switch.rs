//! Suspended SwitchStatement selection, fall-through and scope disposal (14.12).

use crate::common::UnavailableHostRealm;
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
fn awaited_discriminants_and_selectors_run_once_before_selected_fallthrough() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let events=[],result;async function f(){switch(await(events.push('input'),2)){
        case await(events.push('test1'),1):events.push('wrong');break;
        default:events.push('default');break;
        case await(events.push('test2'),2):events.push('match');await 0;
        case await(events.push('skipped'),99):events.push('fallthrough');break;
        }return 7}f().then(value=>{result=value});
    "#,
        )
        .unwrap();
    check(&mut realm, "events.join(',')==='input'&&result===undefined");
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='input,test1,test2,match,fallthrough'&&result===7",
    );
}

#[test]
fn a_middle_default_runs_only_after_all_nonmatching_selectors_and_then_falls_through() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let events=[],result;async function f(){switch(await 8){
        case await(events.push('a'),1):events.push('wrong');break;
        default:events.push('default');await 0;
        case await(events.push('b'),2):events.push('tail');
        }return events.join(',')}f().then(value=>{result=value});
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result==='a,b,default,tail'");
}

#[test]
fn selectors_use_strict_equality_without_coercion_or_implicit_await() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let results=[],conversions=0,key=Symbol('key'),object={[Symbol.toPrimitive](){conversions++;return 1}};
        async function f(input){switch(await input){case object:return 'object';case 1n:return 'bigint';case Promise.resolve(1):return 'wrong';case await key:return 'symbol';case await 1:return 'number';default:return 'default'}}
        f(1).then(x=>results.push(x));f(1n).then(x=>results.push(x));f(key).then(x=>results.push(x));f(object).then(x=>results.push(x));f(NaN).then(x=>results.push(x));
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "conversions===0&&results.includes('number')&&results.includes('bigint')&&results.includes('symbol')&&results.includes('object')&&results.includes('default')&&results.length===5",
    );
}

#[test]
fn discriminants_precede_the_shared_case_scope_and_selectors_observe_its_tdz_and_functions() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let x=9,result,failed,called=0;async function f(){switch(await x){
        case await later():let x=10;await 0;
        case 99:return eval('x');
        default:function later(){called++;return 9}
        }}async function g(){try{switch(await 0){case await hidden:let hidden=0}}catch(error){return error instanceof ReferenceError}}
        f().then(value=>{result=value});g().then(value=>{failed=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===10&&x===9&&called===1&&failed===true&&typeof later==='undefined'&&typeof hidden==='undefined'",
    );
}

#[test]
fn case_body_blocks_dispose_before_fallthrough_in_ordinary_and_suspended_execution() {
    for asynchronous in [false, true] {
        let mut realm = Realm::default();
        realm.eval("let events=[],result;function resource(name){return {[Symbol.dispose](){events.push(name)}}}").unwrap();
        let source = if asynchronous {
            "async function f(){switch(await 1){case 1:{using a=resource('a');await 0;events.push('first')}case 2:{using b=resource('b');events.push('body');break;}default:events.push('wrong')}return events.join(',')}f().then(value=>{result=value})"
        } else {
            "function f(){switch(1){case 1:{using a=resource('a');events.push('first')}case 2:{using b=resource('b');events.push('body');break;}default:events.push('wrong')}return events.join(',')}result=f()"
        };
        realm.eval(source).unwrap();
        drain(&mut realm);
        check(
            &mut realm,
            "result==='first,a,body,b'&&events.join(',')==='first,a,body,b'",
        );
        check(
            &mut realm,
            "eval(\"1;switch(0){case 0:{using c=resource('c');7;break;}}\")===7",
        );
    }
}

#[test]
fn nested_switch_breaks_and_labelled_continues_unwind_after_awaited_finalizers() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result;function resource(name){return {[Symbol.dispose](){events.push(name)}}}
        async function f(){outer:for(let i=0;i<2;i++){switch(await i){case 0:{using a=resource('dispose0');try{switch(await 0){case 0:await 0;continue outer}}finally{await 0;events.push('finally0')}}default:{using b=resource('dispose1');try{await 0;break outer}finally{await 0;events.push('finally1')}}}}return events.join(',')}
        f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result==='finally0,dispose0,finally1,dispose1'");
}

#[test]
fn selector_rejections_restore_scope_and_body_disposal_errors_suppress_the_original_throw() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},disposeReason={},selected=false,failed=false,result,events=[];
        async function f(){try{switch(await 1){case await Promise.reject(reason):selected=true;let hidden=1}}catch(error){failed=error===reason}finally{await 0;events.push('selector-finally')}}
        async function g(){try{switch(await 1){case 1:{using r={[Symbol.dispose](){events.push('dispose');throw disposeReason}};throw await Promise.reject(reason)}}}catch(error){return error instanceof SuppressedError&&error.error===disposeReason&&error.suppressed===reason}finally{await 0;events.push('body-finally')}}
        f();g().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "failed&&!selected&&result===true&&events.includes('selector-finally')&&events.includes('body-finally')&&events.includes('dispose')&&typeof hidden==='undefined'",
    );
}

#[test]
fn selection_retains_the_input_then_releases_it_while_case_lexicals_stay_live() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let select,body,selectGate=new Promise(resolve=>{select=resolve}),bodyGate=new Promise(resolve=>{body=resolve}),inputWeak,localWeak,result;
        function input(){const value={};inputWeak=new WeakRef(value);return value}
        async function f(){switch(input()){case (await selectGate,inputWeak.deref()):let value={x:7};localWeak=new WeakRef(value);await bodyGate;return value.x}}
        f().then(value=>{result=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "inputWeak.deref()!==undefined&&result===undefined",
    );
    realm.eval("select()").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "inputWeak.deref()===undefined&&localWeak.deref()!==undefined&&result===undefined",
    );
    realm.eval("body()").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(&mut realm, "localWeak.deref()===undefined&&result===7");
}

#[test]
fn host_abort_skips_switch_disposal_handlers_and_pending_finalizers() {
    for limited in [false, true] {
        let mut realm = Realm::with_unavailable_host_limits(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},disposed=0,finalized=0,caught=false,proceed,gate=new Promise(resolve=>{proceed=resolve});const dispose=Symbol.dispose;").unwrap();
        realm.eval("let resource={[dispose](){disposed++}};state.weak=new WeakRef(resource[dispose]);function spin(){while(true){}}").unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "spiteUnavailableHost()"
        };
        let code="async function f(){try{switch(await 0){case 0:{using r=resource;await gate;TAIL}}}catch{caught=true}finally{finalized++}}f().catch(()=>{caught=true})".replace("TAIL",tail);
        realm.eval(&code).unwrap();
        drain(&mut realm);
        realm.eval("delete resource[dispose]").unwrap();
        realm.eval("proceed()").unwrap();
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
            "disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}
