//! Ordered Promise.all aggregation, guarded callbacks and traced result lists.

use crate::common::{UnavailableHostRealm, assert_true, drain_promise_jobs};

use spite_runtime::{Error, Limits, Realm, Value};

#[test]
fn results_follow_input_order_and_wait_for_all_values_including_empty_input() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
                let first = Promise.withResolvers(),
                    second = Promise.withResolvers(),
                    seen = [];
                Promise.all([first.promise, second.promise, 3]).then(a => {
                    if (!Array.isArray(a) || a.length !== 3) throw 1;
                    seen.push(a.join(','))
                });
                Promise.all([]).then(a => {
                    if (a.length !== 0) throw 2;
                    seen.push('empty')
                });
                second.resolve(2);
                if (seen.length) throw 3
            "#,
        )
        .unwrap();
    drain_promise_jobs(&mut realm);
    assert_eq!(
        realm.eval("seen.join(';')"),
        Ok(Value::String("empty".into()))
    );
    realm.eval("first.resolve(1)").unwrap();
    drain_promise_jobs(&mut realm);
    assert_eq!(
        realm.eval("seen.join(';')"),
        Ok(Value::String("empty;1,2,3".into()))
    );
    realm
        .eval(
            r#"
                let marker = {},
                    caught = false;
                Promise.all([Promise.reject(marker), new Promise(() => {})]).catch(e => {
                    caught = e === marker
                })
            "#,
        )
        .unwrap();
    drain_promise_jobs(&mut realm);
    assert_eq!(realm.eval("caught"), Ok(Value::Boolean(true)));
    realm
        .eval(
            r#"
                let ArrayIntrinsic = Array,
                    result;
                Array = function() {
                    throw 4
                };
                Object.defineProperty(ArrayIntrinsic.prototype, '0', {
                    set() {
                        throw 5
                    },
                    get() {
                        return 6
                    },
                    configurable: true
                });
                Promise.all([1, 2]).then(a => {
                    result = a
                })
            "#,
        )
        .unwrap();
    drain_promise_jobs(&mut realm);
    assert_eq!(realm.eval("ArrayIntrinsic.isArray(result)&&Object.getPrototypeOf(result)===ArrayIntrinsic.prototype&&result.join(',')==='1,2'"),Ok(Value::Boolean(true)));
}

#[test]
fn generic_resolve_elements_are_guarded_and_delay_resolution_until_iteration_finishes() {
    assert_true(
        r#"
            let log = '',
                callbacks = [],
                result = {},
                sentinel = {},
                resolved = 0,
                output;

            function C(executor) {
                log += 'c';
                executor(function(a) {
                    'use strict';
                    if (this !== undefined || arguments.length !== 1) throw 1;
                    resolved++;
                    output = a;
                    log += 'f';
                    for (const callback of callbacks)
                        if (callback(99) !== undefined) throw 2;
                    return sentinel;
                }, () => {
                    throw 3
                });
                return result;
            }
            Object.defineProperty(C, 'resolve', {
                get() {
                    log += 'g';
                    return function(v) {
                        'use strict';
                        if (this !== C || arguments.length !== 1) throw 4;
                        log += 'r' + v;
                        return {
                            then(callback, reject) {
                                if (arguments.length !== 2 || callback.length !== 1 ||
                                    callback.name !== '' || Object.hasOwn(callback,
                                        'prototype')) throw 5;
                                try {
                                    new callback();
                                    throw 6
                                } catch (e) {
                                    if (!(e instanceof TypeError)) throw e
                                }
                                callbacks.push(callback);
                                if (v !== 3) {
                                    callback(v);
                                    callback(99)
                                }
                                if (resolved !== 0) throw 7;
                            }
                        };
                    };
                }
            });
            let out = Promise.all.call(C, [1, 2, 3]);
            let returned = callbacks[2](3);
            out === result && returned === sentinel && resolved === 1 && output.join(',') === '1,2,3' &&
                log === 'cgr1r2r3f'
        "#,
    );
}

#[test]
fn close_and_rejection_rules_distinguish_body_step_and_exhausted_resolve_failures() {
    assert_true(
        r#"
            let original = {},
                other = {},
                closed = 0,
                caught;

            function C(executor) {
                executor(() => {}, e => {
                    caught = e
                })
            }
            C.resolve = () => {
                throw original
            };
            Promise.all.call(C, {
                [Symbol.iterator]() {
                    return {
                        next() {
                            return {
                                value: 1,
                                done: false
                            }
                        },
                        return () {
                            closed++;
                            throw other
                        }
                    }
                }
            });
            caught === original && closed === 1
        "#,
    );
    assert_true(
        r#"
            let original = {},
                closed = 0,
                caught;

            function C(executor) {
                executor(() => {}, e => {
                    caught = e
                })
            }
            C.resolve = v => v;
            Promise.all.call(C, {
                [Symbol.iterator]() {
                    return {
                        next() {
                            return {
                                done: false,
                                get value() {
                                    throw original
                                }
                            }
                        },
                        return () {
                            closed++;
                            return {}
                        }
                    }
                }
            });
            caught === original && closed === 0
        "#,
    );
    assert_true(
        r#"
            let original = {},
                closed = 0,
                caught;

            function C(executor) {
                executor(() => {
                    throw original
                }, e => {
                    caught = e
                })
            }
            C.resolve = v => v;
            Promise.all.call(C, {
                [Symbol.iterator]() {
                    return {
                        next() {
                            return {
                                done: true
                            }
                        },
                        return () {
                            closed++;
                            return {}
                        }
                    }
                }
            });
            caught === original && closed === 0
        "#,
    );
}

// 27.2.4: the aggregation sentinel survives an abrupt iteration. Subscription
// failures close the iterator; IteratorStepValue failures already mark it done.
#[test]
fn reentrant_callbacks_do_not_complete_aggregates_after_iteration_aborts() {
    for method in ["all", "allSettled", "any", "race"] {
        for stage in ["resolve", "thenGet", "thenCall", "next", "done", "value"] {
            for close_failure in ["none", "get", "call"] {
                let source = r#"
                    let method='METHOD',stage='STAGE',closeFailure='CLOSE';
                    let pairs=[],fulfilled=[],rejected=[],closed=0,index=0;
                    let result={},original={},other={},fired=false;
                    function C(executor){
                        executor(v=>{fulfilled.push(v)},e=>{rejected.push(e)});
                        return result;
                    }
                    function fire(at){
                        if(at!==stage||fired)return;
                        fired=true;
                        pairs[0][0]('first');pairs[0][1]('reason');
                        throw original;
                    }
                    C.resolve=v=>{
                        if(v===2)fire('resolve');
                        return{get then(){
                            if(v===2)fire('thenGet');
                            return(a,b)=>{pairs.push([a,b]);if(v===2)fire('thenCall')};
                        }};
                    };
                    let iterator={
                        next(){
                            if(index===1)fire('next');
                            if(index===2)return{done:true};
                            let v=++index;
                            return{get done(){if(v===2)fire('done');return false},
                                   get value(){if(v===2)fire('value');return v}};
                        },
                        get return(){
                            closed++;
                            if(closeFailure==='get')throw other;
                            return()=>{if(closeFailure==='call')throw other;return{}};
                        }
                    };
                    let out=Promise[method].call(C,{[Symbol.iterator](){return iterator}});
                    for(const pair of pairs){pair[0]('later');pair[1]('laterReason')}
                    out===result&&fired&&pairs.length===(stage==='thenCall'?2:1)&&
                    closed===(['next','done','value'].includes(stage)?0:1)&&
                    rejected.filter(e=>e===original).length===1&&!rejected.includes(other)&&
                    fulfilled.length===(method==='any'||method==='race'?pairs.length+1:0)&&
                    rejected.length===(method==='all'||method==='race'?pairs.length+2:1)
                "#
                .replace("METHOD", method)
                .replace("STAGE", stage)
                .replace("CLOSE", close_failure);
                assert_true(&source);
            }
        }
    }
}

#[test]
fn pending_aggregate_values_and_symbols_survive_collection_and_release_after_completion() {
    let mut realm = Realm::default();
    realm.eval("let state=(()=>{let value={},symbol=Symbol(),cap=Promise.withResolvers();return{all:Promise.all([Promise.resolve(value),Promise.resolve(symbol),cap.promise]),finish:cap.resolve,object:new WeakRef(value),symbol:new WeakRef(symbol)}})()").unwrap();
    drain_promise_jobs(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()!==undefined&&state.symbol.deref()!==undefined"),
        Ok(Value::Boolean(true))
    );
    realm.eval("state.finish(3)").unwrap();
    drain_promise_jobs(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()!==undefined&&state.symbol.deref()!==undefined"),
        Ok(Value::Boolean(true))
    );
    realm.eval("state.all=null;state.finish=null").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()===undefined&&state.symbol.deref()===undefined"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn host_failures_skip_rejection_closing_and_language_handlers() {
    let mut realm = Realm::with_unavailable_host();
    let result = realm.eval(
        r#"
            let closed = 0,
                rejected = 0,
                caught = 0;

            function C(executor) {
                executor(() => {}, () => {
                    rejected++
                })
            }
            C.resolve = () => spiteUnavailableHost();
            try {
                Promise.all.call(C, {
                    [Symbol.iterator]() {
                        return {
                            next() {
                                return {
                                    value: 1,
                                    done: false
                                }
                            },
                            return () {
                                closed++;
                                return {}
                            }
                        }
                    }
                })
            } catch (e) {
                caught++
            } finally {
                caught++
            }
        "#,
    );
    assert!(
        matches!(result, Err(Error::Unsupported { .. })),
        "{result:?}"
    );
    assert_eq!(realm.eval("closed+rejected+caught"), Ok(Value::Number(0.0)));
    let mut limited = Realm::with_unavailable_host_limits(Limits {
        max_steps: Some(100_000),
        ..Limits::default()
    });
    let result = limited.eval(
        r#"
            let closed = 0,
                rejected = 0;

            function C(executor) {
                executor(() => {}, () => {
                    rejected++
                })
            }
            C.resolve = () => {
                while (true) {}
            };
            Promise.all.call(C, {
                [Symbol.iterator]() {
                    return {
                        next() {
                            return {
                                value: 1,
                                done: false
                            }
                        },
                        return () {
                            closed++;
                            return {}
                        }
                    }
                }
            })
        "#,
    );
    assert!(matches!(result, Err(Error::Limit { .. })), "{result:?}");
    assert_eq!(limited.eval("closed+rejected"), Ok(Value::Number(0.0)));
}
