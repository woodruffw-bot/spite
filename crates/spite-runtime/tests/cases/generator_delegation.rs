//! Synchronous yield* iterator forwarding and abrupt completion (15.5).

use spite_runtime::{Error, Limits, PromiseJobOutcome, Realm, Value};

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}
fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn next_is_cached_receives_one_argument_and_forwards_the_exact_result_object() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let calls=[],reads=0,gets=0,pending={get done(){reads++;return false},get value(){throw 'unread'}};
        let source={[Symbol.iterator](){return this},get next(){gets++;return function(value){calls.push([this===source,arguments.length,value]);return calls.length===1?pending:{done:true,value:7}}}};
        function*g(){return (yield* source)+1}let iterator=g(),first=iterator.next(99);Object.defineProperty(source,'next',{value:()=>{throw 'reloaded'}});let last=iterator.next(5);
        first===pending&&last.done&&last.value===8&&reads===1&&gets===1&&calls[0][0]&&calls[0][1]===1&&calls[0][2]===undefined&&calls[1][0]&&calls[1][1]===1&&calls[1][2]===5
    "#,
    );
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        PromiseJobOutcome::NoJob
    ));
}

#[test]
fn throw_uses_live_method_lookup_and_its_done_value_completes_the_expression() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let reason={},reads=0,calls=0,pending={done:false,value:3};let source={[Symbol.iterator](){return this},next(){return pending}};
        function*g(){return (yield*source)+1}let iterator=g();iterator.next();
        Object.defineProperty(source,'throw',{get(){reads++;return function(value){calls++;return {done:true,value:this===source&&arguments.length===1&&value===reason?7:0}}}});
        let result=iterator.throw(reason);result.done&&result.value===8&&reads===1&&calls===1
    "#,
    );
}

#[test]
fn return_done_false_yields_then_next_resumes_normally_and_done_true_returns() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let after=0,events=[],pending={done:false,value:2},source={[Symbol.iterator](){return this},next(value){events.push(value);return value===9?{done:true,value:5}:pending},return(value){events.push('return:'+value);return pending}};
        function*g(){try{let x=yield*source;after++;return x+1}finally{events.push('finally')}}
        let a=g();a.next();let paused=a.return(7),last=a.next(9);source.return=function(value){return {done:true,value:value+3}};
        let b=g();b.next();let returned=b.return(11);
        paused===pending&&last.done&&last.value===6&&returned.done&&returned.value===14&&after===1&&events.join(',')===',return:7,9,finally,,finally'
    "#,
    );
}

#[test]
fn absent_or_null_return_preserves_the_original_return_value_without_awaiting() {
    for returned in ["undefined", "null"] {
        let mut realm = Realm::default();
        check(
            &mut realm,
            &format!(
                r#"
            let after=0,promise=Promise.resolve(7),source={{[Symbol.iterator](){{return this}},next(){{return{{done:false,value:1}}}},return:{returned}}};
            function*g(){{yield*source;after++}}let iterator=g();iterator.next();let result=iterator.return(promise);
            result.done&&result.value===promise&&after===0
        "#
            ),
        );
    }
}

#[test]
fn missing_throw_closes_with_no_arguments_and_close_errors_precede_protocol_typeerror() {
    for close in ["return {}", "throw closing", "return 1"] {
        let mut realm = Realm::default();
        let expected = if close == "throw closing" {
            "error===closing"
        } else {
            "error instanceof TypeError"
        };
        check(
            &mut realm,
            &format!(
                r#"
            let closing={{}},reason={{}},calls=0,receiver=false,arity=-1,source={{[Symbol.iterator](){{return this}},next(){{return{{done:false,value:1}}}},return(){{calls++;receiver=this===source;arity=arguments.length;{close}}}}};
            function*g(){{yield*source}}let iterator=g();iterator.next();let same=false;try{{iterator.throw(reason)}}catch(error){{same={expected}}}
            same&&calls===1&&receiver&&arity===0&&iterator.next().done
        "#
            ),
        );
    }
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let closed=0,source={[Symbol.iterator](){return this},next(){return{done:false}},throw:1,return(){closed++;return{}}};
        function*g(){yield*source}let iterator=g();iterator.next();let same=false;try{iterator.throw(7)}catch(error){same=error instanceof TypeError}same&&closed===0
    "#,
    );
}

#[test]
fn result_done_and_final_value_failures_propagate_without_automatic_iterator_close() {
    for result in [
        "1",
        "{get done(){throw reason}}",
        "{done:true,get value(){throw reason}}",
    ] {
        let mut realm = Realm::default();
        let expected = if result == "1" {
            "error instanceof TypeError"
        } else {
            "error===reason"
        };
        check(
            &mut realm,
            &format!(
                r#"
            let reason={{}},closed=0,source={{[Symbol.iterator](){{return this}},next(){{return {result}}},return(){{closed++;return{{}}}}}};
            function*g(){{try{{yield*source}}catch(error){{return {expected}}}}}let iterator=g(),result=iterator.next();
            result.done&&result.value===true&&closed===0&&iterator.next().done
        "#
            ),
        );
    }
}

#[test]
fn nested_delegation_and_yielded_operands_share_surrounding_expression_state() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        function*inner(){yield 1;return 7}function*outer(){let x=yield*inner();return `${x}:${yield*(yield 'source')}`}
        let iterator=outer(),a=iterator.next(),b=iterator.next(),c=iterator.next([3,4]),d=iterator.next(),e=iterator.next();
        a.value===1&&b.value==='source'&&c.value===3&&d.value===4&&e.done&&e.value==='7:undefined'&&[...(function*(){yield*'a😀'})()].join(',')==='a,😀'
    "#,
    );
}

#[test]
fn delegate_reentry_uses_the_executing_brand_and_restores_the_caller_context() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let reentry=0,source={[Symbol.iterator](){return this},next(){try{iterator.next()}catch(error){if(error instanceof TypeError)reentry++}return{done:false,value:7}},return(){return{done:true,value:9}}};
        function*g(){yield*source}let iterator=g(),a=iterator.next(),b=iterator.return(3);a.value===7&&b.done&&b.value===9&&reentry===1
    "#,
    );
}

#[test]
fn suspended_delegates_retain_iterator_and_cached_next_then_release_them() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let nextWeak,sourceWeak,iterator=(()=>{let source={[Symbol.iterator](){return this},next(){return{done:false,value:1}},return(){return{done:true}}};sourceWeak=new WeakRef(source);nextWeak=new WeakRef(source.next);function*g(){yield*source}let iterator=g();iterator.next();delete source.next;return iterator})();
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "sourceWeak.deref()!==undefined&&nextWeak.deref()!==undefined",
    );
    realm.eval("iterator.return(7)").unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "sourceWeak.deref()===undefined&&nextWeak.deref()===undefined",
    );
}

#[test]
fn delegate_host_failures_complete_and_release_state_without_cleanup() {
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm
            .eval("let calls=0,closed=0,caught=0,finalized=0;function spin(){while(true){}}")
            .unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "Function('async function* gap(){}')"
        };
        realm.eval(&format!(r#"
            let weak;function*g(){{let source={{[Symbol.iterator](){{return this}},next(){{if(calls++){{{tail}}}return{{done:false,value:1}}}},return(){{closed++;return{{}}}}}};weak=new WeakRef(source);try{{yield*source}}catch{{caught++}}finally{{finalized++}}}}let iterator=g();iterator.next()
        "#)).unwrap();
        let error = realm.eval("iterator.next()").unwrap_err();
        assert!(
            if limited {
                matches!(error, Error::Limit { .. })
            } else {
                matches!(error, Error::Unsupported { .. })
            },
            "{error}"
        );
        collect(&mut realm);
        check(
            &mut realm,
            "closed===0&&caught===0&&finalized===0&&iterator.next().done&&weak.deref()===undefined",
        );
    }
}
