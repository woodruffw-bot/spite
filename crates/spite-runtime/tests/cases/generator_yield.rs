//! Ordinary GeneratorYield and resumed completions through the source VM (27.5.3).

use spite_runtime::{Error, Limits, PromiseJobOutcome, Realm, Value};
fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}
fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn next_supplies_expression_values_and_bare_yield_without_promise_jobs() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        function*g(){let x=yield 1;yield;return x+(yield 3)}let iterator=g(),a=iterator.next(99),b=iterator.next(7),c=iterator.next(8),d=iterator.next(9);
        a.value===1&&!a.done&&b.value===undefined&&!b.done&&c.value===3&&!c.done&&d.value===16&&d.done&&iterator.next().done
    "#,
    );
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::NoJob
    ));
}

#[test]
fn nested_operands_member_targets_and_templates_resume_without_replaying_effects() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let events=[],target={};function*g(){let values=[(events.push('a'),yield 1),(events.push('b'),yield 2)];target[yield 3]=yield 4;return `${values.join(',')}:${target.x}:${yield 5}`}
        let iterator=g(),a=iterator.next(),b=iterator.next(7),c=iterator.next(9),d=iterator.next('x'),e=iterator.next(11),f=iterator.next(13);
        [a.value,b.value,c.value,d.value,e.value].join(',')==='1,2,3,4,5'&&f.done&&f.value==='7,9:11:13'&&events.join(',')==='a,b'
    "#,
    );
}

#[test]
fn loops_labels_switch_and_with_restore_their_progress_and_scopes() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        function*g(){let out=[];outer:for(let i=0;i<3;i++){if(i===1)continue outer;out.push(yield i)}let n=0;while(yield n++){break}do{out.push(yield 'do')}while(false);with({x:7}){yield x}switch(yield 'switch'){case 3:yield 'hit';break}return out.join(',')}
        let iterator=g(),values=[iterator.next().value,iterator.next(7).value,iterator.next(9).value,iterator.next(false).value,iterator.next(11).value,iterator.next().value,iterator.next(3).value],last=iterator.next();
        values.join(',')==='0,2,0,do,7,switch,hit'&&last.done&&last.value==='7,9,11'&&typeof x==='undefined'
    "#,
    );
}

#[test]
fn injected_throw_reaches_catches_and_finally_and_uncaught_values_keep_identity() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let reason={},events=[];function*g(){try{return yield 1}catch(error){yield error;return 7}finally{events.push('finally')}}let a=g();a.next();let caught=a.throw(reason),last=a.next();
        let b=(function*(){yield 1})();b.next();let same=false;try{b.throw(reason)}catch(error){same=error===reason}
        caught.value===reason&&!caught.done&&last.done&&last.value===7&&events.join(',')==='finally'&&same&&b.next().done
    "#,
    );
}

#[test]
fn pending_returns_survive_yielding_finalizers_and_later_abrupts_can_override_them() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let events=[],reason={};function*g(){try{yield 1;events.push('wrong')}finally{yield 2;events.push('finished')}}
        let a=g();a.next();let paused=a.return(7),last=a.next(9);let b=g();b.next();b.return(7);let overridden=b.return(11);
        let c=g();c.next();c.return(7);let thrown=false;try{c.throw(reason)}catch(error){thrown=error===reason}
        !paused.done&&paused.value===2&&last.done&&last.value===7&&overridden.done&&overridden.value===11&&thrown&&c.next().done&&events.join(',')==='finished'
    "#,
    );
}

#[test]
fn return_yield_and_short_circuits_observe_only_selected_suspensions() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let effects=0;function*g(){false&&(yield missing);true||(yield missing);return yield (effects++,7)}let iterator=g(),first=iterator.next(),last=iterator.next(13);
        first.value===7&&!first.done&&last.value===13&&last.done&&effects===1
    "#,
    );
}

#[test]
fn array_object_var_and_catch_binding_defaults_use_the_shared_pattern_cursor() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        function*g(){let {[yield 'key']:x=yield 'default'}={};var[y=yield 'array']=[];try{throw []}catch([z=yield 'catch']){yield [x,y,z];return x+y+z}}
        let iterator=g(),a=iterator.next(),b=iterator.next('x'),c=iterator.next(7),d=iterator.next(9),e=iterator.next(11),last=iterator.next();
        [a.value,b.value,c.value,d.value].join(',')==='key,default,array,catch'&&e.value.join(',')==='7,9,11'&&!e.done&&last.done&&last.value===27
    "#,
    );
}

#[test]
fn abrupt_binding_resumes_close_nested_iterators_inside_out_with_correct_precedence() {
    for closing_error in [false, true] {
        for thrown in [false, true] {
            let mut realm = Realm::default();
            realm.eval("let reason={},closing={},events=[];function source(name,value){return{[Symbol.iterator](){return this},next(){return{done:false,value}},return(){events.push(name);if(fail)throw closing;return{}}}}let fail=false;").unwrap();
            realm.eval("function*g(){let[[x=yield 1]]=source('outer',source('inner',undefined))}let iterator=g();iterator.next();").unwrap();
            if closing_error {
                realm.eval("fail=true").unwrap();
            }
            if thrown {
                check(
                    &mut realm,
                    "let same=false;try{iterator.throw(reason)}catch(error){same=error===reason}same&&iterator.next().done",
                );
            } else if closing_error {
                check(
                    &mut realm,
                    "let same=false;try{iterator.return(9)}catch(error){same=error===closing}same&&iterator.next().done",
                );
            } else {
                check(
                    &mut realm,
                    "let result=iterator.return(9);result.done&&result.value===9",
                );
            }
            check(&mut realm, "events.join(',')==='inner,outer'");
        }
    }
}

#[test]
fn for_of_closes_before_yielding_finally_and_awaited_style_targets_keep_input_values() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let events=[],target={},source={[Symbol.iterator](){return this},next(){return{done:false,value:7}},return(){events.push('close');return{}}};
        function*g(){try{for(target[yield 'key']of source){yield target.x}}finally{yield 'finally'}}let iterator=g(),a=iterator.next(),b=iterator.next('x'),c=iterator.return(9),d=iterator.next();
        a.value==='key'&&b.value===7&&c.value==='finally'&&!c.done&&d.done&&d.value===9&&events.join(',')==='close'&&target.x===7
    "#,
    );
}

#[test]
fn method_private_and_argument_contexts_survive_yield_and_collection() {
    let mut realm = Realm::default();
    realm.eval("let weak;class C{#x=7;*g(a){yield a;return this.#x+arguments[0]}}let object=new C;weak=new WeakRef(object);let iterator=object.g(3);object=null;iterator.next()").unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()!==undefined&&iterator.next().value===10",
    );
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined");
}

#[test]
fn yielded_state_retains_callees_and_locals_and_releases_them_after_completion() {
    let mut realm = Realm::default();
    realm.eval("let weak,calleeWeak,iterator=(()=>{let kept={};weak=new WeakRef(kept);let f=function*(arguments){kept;yield 1;return arguments};calleeWeak=new WeakRef(f);let result=f(7);f=null;return result})();iterator.next()").unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()!==undefined&&calleeWeak.deref()!==undefined",
    );
    check(&mut realm, "iterator.next().value===7");
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()===undefined&&calleeWeak.deref()===undefined",
    );
}

#[test]
fn host_abort_after_a_yield_and_unimplemented_delegation_skip_remaining_cleanup() {
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm
            .eval("let effects=0,disposed=0,finalized=0,caught=0;function spin(){while(true){}}")
            .unwrap();
        realm.eval("let resource={[Symbol.dispose](){disposed++}},weak=new WeakRef(resource[Symbol.dispose])").unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "yield* (effects++)"
        };
        realm.eval(&format!("function*g(){{using r=resource;delete resource[Symbol.dispose];try{{yield 1;{tail}}}catch{{caught++}}finally{{finalized++}}}}let iterator=g();iterator.next()")).unwrap();
        let error = realm.eval("iterator.next()").unwrap_err();
        if limited {
            assert!(matches!(error, Error::Limit { .. }), "{error}");
        } else {
            assert!(matches!(error, Error::Unsupported { .. }), "{error}");
        }
        collect(&mut realm);
        check(
            &mut realm,
            "effects===0&&disposed===0&&finalized===0&&caught===0&&iterator.next().done&&weak.deref()===undefined",
        );
    }
}

#[test]
fn sync_and_async_consumers_close_yielded_generators_and_preserve_values() {
    let mut realm = Realm::default();
    realm.eval("let events=[],result;function*g(){try{yield 1;yield 2}finally{events.push('close')}}let values=[...g()];for(const x of g())break;async function f(){let out=[];for await(const x of g())out.push(x);return out.join(',')}f().then(value=>{result=value})").unwrap();
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
    check(
        &mut realm,
        "values.join(',')==='1,2'&&result==='1,2'&&events.join(',')==='close,close,close'",
    );
}

#[test]
fn yielded_symbols_bigints_and_promises_remain_raw_values() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        "let key=Symbol(),promise=Promise.resolve(7);function*g(){yield key;yield 9n;return yield promise}let iterator=g(),a=iterator.next(),b=iterator.next(),c=iterator.next(),d=iterator.next(key);a.value===key&&b.value===9n&&c.value===promise&&!c.done&&d.done&&d.value===key",
    );
}

#[test]
fn bare_yield_inside_template_middle_and_tail_resumes_its_substitution() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let effects=0;function*g(){return `a${yield}b${(effects++,yield)}c`}
        let iterator=g(),a=iterator.next(),b=iterator.next(7),c=iterator.next(9);
        !a.done&&a.value===undefined&&!b.done&&b.value===undefined&&c.done&&c.value==='a7b9c'&&effects===1
    "#,
    );
}
