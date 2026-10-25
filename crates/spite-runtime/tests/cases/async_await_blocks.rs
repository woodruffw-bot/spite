//! Lexical BlockStatement suspension, disposal and collection.

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

#[test]
fn nested_block_bindings_keep_their_scopes_and_var_bindings_target_the_function() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],result;state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){let value=1;{let value=await state.gate;var saved=value;{const[x,y]=await[2,3];events.push(value+x+y)}events.push(value)}events.push(value);return saved}
        f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "events.length===0&&result===undefined");
    realm.eval("state.release(7)").unwrap();
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='12,7,1'&&result===7");
}

#[test]
fn block_instantiation_preserves_tdz_and_strict_block_function_hoisting() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let before,outside,tdz=false,result;
        async function f(){'use strict';{before=typeof local;const read=()=>value;try{read()}catch(error){tdz=error instanceof ReferenceError}const value=await 3;function local(){return value}result=local()+read()}outside=typeof local}
        f();
    "#).unwrap();
    check(&mut realm, "before==='function'&&tdz&&result===undefined");
    drain(&mut realm);
    check(&mut realm, "result===6&&outside==='undefined'");
}

#[test]
fn normal_block_exit_disposes_before_parent_statements_and_root_disposal() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},events=[],result;state.gate=new Promise(resolve=>{state.release=resolve});
        async function f(){using r={[Symbol.dispose](){events.push('root')}};{using a={[Symbol.dispose](){events.push('a')}};await state.gate;{using b={[Symbol.dispose](){events.push('b')}};await 0}events.push('after-inner')}events.push('after-outer');return 7}
        f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "events.length===0&&result===undefined");
    realm.eval("state.release(0)").unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='b,after-inner,a,after-outer,root'&&result===7",
    );
}

#[test]
fn returns_throws_and_rejected_awaits_unwind_all_blocks_with_ordered_suppression() {
    for (mode, tail) in [
        (0, "return await 7"),
        (1, "throw await marker"),
        (2, "await Promise.reject(marker)"),
    ] {
        let mut realm = Realm::default();
        realm.eval(&format!("let mode={mode};")).unwrap();
        let source=r#"
            let marker={},r={},a={},b={},events=[],result,reason,later=0;
            async function f(){
                using root={[Symbol.dispose](){events.push('root');if(mode===1)throw r}};
                {using outer={[Symbol.dispose](){events.push('a');if(mode===1)throw a}};
                    {using inner={[Symbol.dispose](){events.push('b');if(mode===1)throw b}};TAIL;later++}
                }
            }
            f().then(value=>{result=value},value=>{reason=value});
        "#.replace("TAIL",tail);
        realm.eval(&source).unwrap();
        drain(&mut realm);
        check(&mut realm, "events.join(',')==='b,a,root'&&later===0");
        check(
            &mut realm,
            match mode {
                0 => "result===7&&reason===undefined",
                1 => {
                    "reason instanceof SuppressedError&&reason.error===r&&reason.suppressed.error===a&&reason.suppressed.suppressed.error===b&&reason.suppressed.suppressed.suppressed===marker"
                }
                _ => "reason===marker&&result===undefined",
            },
        );
    }
}

#[test]
fn exited_block_bindings_are_collectible_while_the_function_remains_suspended() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let state={},result;state.first=new Promise(resolve=>{state.releaseFirst=resolve});state.second=new Promise(resolve=>{state.releaseSecond=resolve});
        async function f(){let saved;{const value={x:7};state.weak=new WeakRef(value);await state.first;saved=value.x}await state.second;return saved}
        f().then(value=>{result=value});
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.weak.deref()!==undefined&&result===undefined",
    );
    realm.eval("state.releaseFirst(0)").unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "state.weak.deref()===undefined&&result===undefined",
    );
    realm.eval("state.releaseSecond(0)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===7");
}

#[test]
fn class_private_eval_this_and_arguments_survive_nested_block_awaits() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result;
        class C{#x=10;async read(value){{const x=await value;{await 0;return eval('this.#x')+x+arguments.length}}}}
        new C().read(2).then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result===13");
}

#[test]
fn host_abort_clears_all_block_disposers_without_running_them_even_at_budget_zero() {
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        let tail = if limited {
            "while(true){}"
        } else {
            "new SharedArrayBuffer(0)"
        };
        // Each setup turn uses the same allowance as the resumed execution.
        realm.eval("let state={},disposed=0,caught=false,read;const dispose=Symbol.dispose;state.weaks=[];").unwrap();
        for name in ["root", "outer", "inner"] {
            realm
                .eval(&format!("let {name}={{[dispose](){{disposed++}}}};"))
                .unwrap();
            realm
                .eval(&format!("state.weaks.push(new WeakRef({name}[dispose]));"))
                .unwrap();
        }
        realm.eval(&"async function f(){using r=root;{using a=outer;{using b=inner;read=()=>b;await 0;TAIL}}}".replace("TAIL",tail)).unwrap();
        realm.eval("f().catch(()=>{caught=true});").unwrap();
        realm
            .eval("delete root[dispose];delete outer[dispose];delete inner[dispose];")
            .unwrap();
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
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        check(
            &mut realm,
            "disposed===0&&!caught&&read()===inner&&state.weaks.every(value=>value.deref()===undefined)",
        );
    }
}
