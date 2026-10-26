//! Asynchronous ForIn/OfBodyEvaluation and AsyncIteratorClose (14.7.5, 7.4).

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
fn sync_values_are_unwrapped_while_async_iterator_values_stay_raw() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let promise=Promise.resolve(7),sync=[],raw=[],done=0;
        async function a(){for await(const x of await [promise,9]){sync.push(x)}}
        let source={i:0,[Symbol.asyncIterator](){return this},next(){return Promise.resolve(this.i++===0?{value:promise,done:false}:{done:true})}};
        async function b(){for await(const x of source){raw.push(x)}}a().then(()=>{done++});b().then(()=>{done++});
    "#).unwrap();
    check(&mut realm, "sync.length===0&&raw.length===0&&done===0");
    drain(&mut realm);
    check(
        &mut realm,
        "done===2&&sync.join(',')==='7,9'&&raw.length===1&&raw[0]===promise",
    );
}

#[test]
fn pending_next_calls_are_sequential_and_cache_the_method_and_receiver() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),calls=0,reads=0,closed=0,values=[],result;
        let iterator={get next(){reads++;return function(){calls++;if(this!==iterator)throw 1;return calls===1?gate:{done:true}}},return(){closed++;return{done:true}}};
        async function f(){for await(const x of {[Symbol.asyncIterator](){return iterator}}){values.push(x);await 0}return 9}f().then(value=>{result=value});
    "#).unwrap();
    check(
        &mut realm,
        "calls===1&&reads===1&&values.length===0&&result===undefined",
    );
    realm.eval("Object.defineProperty(iterator,'next',{value(){throw 2}});release({value:7,done:false})").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "calls===2&&reads===1&&closed===0&&values.join(',')==='7'&&result===9",
    );
}

#[test]
fn async_methods_take_precedence_and_nullish_methods_fall_back_to_sync_iteration() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let done=0,asyncCalls=0,syncCalls=0,failed=0,values=[];
        let preferred={get [Symbol.iterator](){throw 1},[Symbol.asyncIterator](){asyncCalls++;return{next(){return{done:true}}}}};
        let fallback={[Symbol.asyncIterator]:null,[Symbol.iterator](){syncCalls++;return[7][Symbol.iterator]()}};
        async function f(value){for await(const x of value){values.push(x)}}
        f(preferred).then(()=>{done++});f(fallback).then(()=>{done++});f('a').then(()=>{done++});
        for(const value of [null,1,{[Symbol.asyncIterator]:0,[Symbol.iterator](){throw 2}}])f(value).then(undefined,error=>{if(error instanceof TypeError)failed++});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "done===3&&failed===3&&asyncCalls===1&&syncCalls===1&&values.includes(7)&&values.includes('a')&&values.length===2",
    );
}

#[test]
fn next_rejections_and_result_getter_errors_do_not_close_the_iterator() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},closed=0,results=[];
        function source(kind){return{[Symbol.asyncIterator](){return this},next(){
        if(kind==='call')throw reason;if(kind==='reject')return Promise.reject(reason);if(kind==='primitive')return 1;
        let result={done:false,value:7};if(kind==='done')Object.defineProperty(result,'done',{get(){throw reason}});if(kind==='value')Object.defineProperty(result,'value',{get(){throw reason}});
        if(kind==='constructor'){let promise=Promise.resolve(result);Object.defineProperty(promise,'constructor',{get(){throw reason}});return promise}return result},return(){closed++;return{done:true}}}}
        async function f(kind){try{for await(const x of source(kind)){}return false}catch(error){return kind==='primitive'?error instanceof TypeError:error===reason}}
        for(const kind of ['call','reject','primitive','done','value','constructor'])f(kind).then(value=>results.push(value));
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===6&&results.every(x=>x===true)&&closed===0",
    );
}

#[test]
fn break_and_return_wait_for_close_without_reading_its_result_properties() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),events=[],result;
        let iterator={[Symbol.asyncIterator](){return this},next(){return{value:7,done:false}},return(){events.push(this===iterator&&arguments.length===0?'close':'wrong');return gate}};
        async function f(){for await(const x of iterator){try{return x}finally{await 0;events.push('finally')}}}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===undefined&&events.join(',')==='finally,close'",
    );
    realm
        .eval("release({get done(){throw 1},get value(){throw 2}})")
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7&&events.join(',')==='finally,close'");
}

#[test]
fn closing_language_errors_override_break_but_preserve_an_original_throw() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let original={},closing={},results=[];
        function source(kind){let iterator={[Symbol.asyncIterator](){return this},next(){return{done:false,value:0}}};Object.defineProperty(iterator,'return',{get(){if(kind==='get')throw closing;if(kind==='noncall')return 0;return function(){if(kind==='call')throw closing;if(kind==='reject')return Promise.reject(closing);return 1}}});return iterator}
        async function f(kind,throwing){try{for await(const x of source(kind)){if(throwing)throw original;break}return false}catch(error){return throwing?error===original:(kind==='noncall'||kind==='primitive'?error instanceof TypeError:error===closing)}}
        for(const kind of ['get','noncall','call','reject','primitive'])for(const throwing of [false,true])f(kind,throwing).then(value=>results.push(value));
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===10&&results.every(x=>x===true)",
    );
}

#[test]
fn return_lookup_is_live_and_absent_or_null_methods_allow_abrupt_completion() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,events=[],iterator={[Symbol.asyncIterator](){return this},next(){return{done:false,value:7}},return(){throw 1}};
        async function f(){for await(const x of iterator){await 0;iterator.return=function(){events.push('late');return{done:true}};break}for await(const x of {[Symbol.asyncIterator](){return this},next(){return{done:false,value:0}},return:null})break;return 9}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===9&&events.join(',')==='late'");
}

#[test]
fn labelled_continues_keep_their_iterator_open_and_close_intervening_iterators() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result;function source(name){return{i:0,[Symbol.asyncIterator](){return this},next(){return this.i++<2?{done:false,value:this.i}:{done:true}},return(){events.push(name);return Promise.resolve({done:true})}}}
        async function f(){outer:for await(const x of source('outer')){for await(const y of source('inner')){try{await 0;if(x===1)continue outer;break outer}finally{await 0;events.push('finally')}}}return events.join(',')}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result==='finally,inner,finally,inner,outer'");
}

#[test]
fn lexical_tdz_fresh_closures_patterns_and_var_parameter_aliases_use_shared_bindings() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let x=[7],tdz=false,closures=[],result,alias;
        async function f(){try{for await(let x of x){}}catch(error){tdz=error instanceof ReferenceError}for await(const [x=3] of [[7],[]]){closures.push(()=>x);await 0}return closures.map(f=>f()).join(',')}
        async function g(x){for await(var x of [7,9]){}return arguments[0]}f().then(value=>{result=value});g(1).then(value=>{alias=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "tdz&&result==='7,3'&&alias===9&&x[0]===7");
}

#[test]
fn using_heads_dispose_before_close_and_use_synchronous_disposal() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result,empty=false;function resource(name){return{name,[Symbol.dispose](){events.push('dispose'+name)},get [Symbol.asyncDispose](){throw 1}}}
        let iterator={i:0,[Symbol.iterator](){return this},next(){return{done:false,value:Promise.resolve(resource(++this.i))}},return(){events.push('close');return{done:true}}};
        async function f(){for await(using r of iterator){events.push('body'+r.name);await 0;if(r.name===2)break}return events.join(',')}f().then(value=>{result=value});
        async function g(){for await(using x of []){throw 2}}g().then(()=>{empty=true});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result==='body1,dispose1,body2,dispose2,close'&&empty",
    );
}

#[test]
fn binding_failures_and_disposal_errors_close_after_lexical_scope_unwinding() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let original={},disposal={},events=[],binding=false,result;
        function source(value){return{[Symbol.asyncIterator](){return this},next(){return{done:false,value}},return(){events.push('close');return{done:true}}}}
        async function f(){try{for await(const [x] of source(null)){} }catch(error){return error instanceof TypeError}}
        async function g(){try{for await(using r of source({[Symbol.dispose](){events.push('dispose');throw disposal}})){try{throw await Promise.reject(original)}finally{await 0;events.push('finally')}}}catch(error){return error instanceof SuppressedError&&error.error===disposal&&error.suppressed===original}}
        f().then(value=>{binding=value});g().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "binding&&result===true&&events.filter(x=>x==='close').length===2&&events.indexOf('finally')<events.indexOf('dispose')&&events[events.length-1]==='close'",
    );
}

#[test]
fn pending_iteration_retains_the_iterator_and_cached_next_then_releases_both() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),weak,nextWeak,result;
        function source(){let first=true;const iterator={next(){if(first){first=false;return gate}return{done:true}}};weak=new WeakRef(iterator);nextWeak=new WeakRef(iterator.next);return{[Symbol.asyncIterator](){return iterator}}}
        async function f(){for await(const x of source()){weak.deref().next=null;await 0}return 7}f().then(value=>{result=value});
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()!==undefined&&nextWeak.deref()!==undefined&&result===undefined",
    );
    realm.eval("release({done:false,value:1})").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "result===7&&weak.deref()===undefined&&nextWeak.deref()===undefined",
    );
}

#[test]
fn host_abort_during_next_body_or_close_bypasses_handlers_finalizers_and_disposal() {
    for phase in ["next", "body", "close"] {
        for limited in [false, true] {
            let mut realm = Realm::new(Limits {
                max_steps: limited.then_some(10_000),
                ..Default::default()
            });
            realm.eval("let state={},disposed=0,finalized=0,caught=false,proceed,gate=new Promise(resolve=>{proceed=resolve}),original={};const dispose=Symbol.dispose;").unwrap();
            realm.eval("let resource={[dispose](){disposed++}};state.weak=new WeakRef(resource[dispose]);function spin(){while(true){}}").unwrap();
            let tail = if limited {
                "spin()"
            } else {
                "new SharedArrayBuffer(0)"
            };
            let next = if phase == "next" {
                tail
            } else {
                "({done:false,value:0})"
            };
            let body = if phase == "body" {
                tail
            } else {
                "throw original"
            };
            let close = if phase == "close" {
                tail
            } else {
                "({done:true})"
            };
            let source = format!(
                "let iterator={{[Symbol.asyncIterator](){{return this}},next(){{return {next}}},return(){{return {close}}}}};async function f(){{using r=resource;await gate;try{{for await(const x of iterator){{{body}}}}}catch{{caught=true}}finally{{finalized++}}}}f().catch(()=>{{caught=true}})"
            );
            realm.eval(&source).unwrap();
            realm.eval("delete resource[dispose];proceed()").unwrap();
            let mut reported = false;
            loop {
                match realm.run_next_promise_job().unwrap() {
                    PromiseJobOutcome::NoJob => break,
                    PromiseJobOutcome::Completed => (),
                    PromiseJobOutcome::Reported(error) => {
                        assert!(
                            matches!(error, Error::Limit { .. }) == limited,
                            "{phase}: {error}"
                        );
                        if !limited {
                            assert!(matches!(error, Error::Unsupported { .. }), "{error}")
                        }
                        reported = true;
                    }
                }
            }
            assert!(reported, "{phase}");
            collect(&mut realm);
            check(
                &mut realm,
                "disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
            );
        }
    }
}
