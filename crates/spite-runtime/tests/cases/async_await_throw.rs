//! Real source throw completions after Await and expression continuation.

use crate::common::{UnavailableHostRealm, assert_true_in, drain_promise_jobs};

use spite_runtime::{Error, PromiseJobOutcome, Realm};

#[test]
fn throw_await_waits_for_fulfillment_and_preserves_await_rejection_reasons() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
                let state={},value={},marker={},reason,rejected,later=0;
                state.gate=new Promise(resolve=>{state.release=resolve});
                async function f(){throw await state.gate;later++}
                async function g(){throw await Promise.reject(marker)}
                f().catch(value=>{reason=value});g().catch(value=>{rejected=value});
            "#,
        )
        .unwrap();
    assert_true_in(
        &mut realm,
        "reason===undefined&&rejected===undefined&&later===0",
    );
    drain_promise_jobs(&mut realm);
    assert_true_in(
        &mut realm,
        "reason===undefined&&rejected===marker&&later===0",
    );
    realm.eval("state.release(value)").unwrap();
    drain_promise_jobs(&mut realm);
    assert_true_in(&mut realm, "reason===value&&rejected===marker&&later===0");
}

#[test]
fn awaited_throw_disposes_resources_in_reverse_and_preserves_suppression_chains() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
                let marker = {},
                    first = {},
                    second = {},
                    events = [],
                    reason;
                async function f() {
                    using a = {
                        [Symbol.dispose]() {
                            events.push('a');
                            throw first
                        }
                    }, b = {
                        [Symbol.dispose]() {
                            events.push('b');
                            throw second
                        }
                    };
                    throw await marker;
                }
                f().catch(value => {
                    reason = value
                });
            "#,
        )
        .unwrap();
    assert_true_in(&mut realm, "events.length===0&&reason===undefined");
    drain_promise_jobs(&mut realm);
    assert_true_in(
        &mut realm,
        r#"
            events.join(',') === 'b,a' && reason instanceof SuppressedError && reason.error === first &&
                reason.suppressed instanceof SuppressedError && reason.suppressed.error === second &&
                reason.suppressed.suppressed === marker
        "#,
    );
}

#[test]
fn completed_throw_expressions_reject_raw_thenables_and_preserve_symbol_identity() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
                let calls=0,thenable={then(){calls++}},symbol=Symbol(),first,second;
                async function f(){throw(await 0,thenable)}
                async function g(){throw await symbol}
                f().catch(value=>{first=value});g().catch(value=>{second=value});
            "#,
        )
        .unwrap();
    drain_promise_jobs(&mut realm);
    assert_true_in(&mut realm, "first===thenable&&second===symbol&&calls===0");
}

#[test]
fn host_failure_in_resumed_throw_operands_bypasses_rejection_and_releases_cached_disposers() {
    let mut realm = Realm::with_unavailable_host();
    realm
        .eval(
            r#"
                let state={},disposed=0,caught=false,read,resource={[Symbol.dispose](){disposed++}};
                state.weak=new WeakRef(resource[Symbol.dispose]);
                async function f(){using r=resource;read=()=>r;throw(await 0,spiteUnavailableHost())}
                f().catch(()=>{caught=true});delete resource[Symbol.dispose];
            "#,
        )
        .unwrap();
    let mut reported = false;
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(Error::Unsupported { .. }) => reported = true,
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
    assert!(reported);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_true_in(
        &mut realm,
        "disposed===0&&!caught&&state.weak.deref()===undefined&&read()===resource",
    );
}
