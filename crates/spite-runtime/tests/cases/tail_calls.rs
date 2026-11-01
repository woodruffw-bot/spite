//! Strict tail transfers and the cleanup that prevents early caller retirement.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn strict_functions_arrows_methods_and_forwarders_reuse_native_call_depth() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            check(
                r#"
            'use strict';
            function even(n){if(n===0)return true;return odd(n-1)}
            function odd(n){if(n===0)return false;return even(n-1)}
            let arrow=n=>n===0?7:arrow(n-1);
            class C{m(n){if(n===0)return this;return this.m(n-1)}
                #private(n){if(n===0)return this;return this.#private(n-1)}
                private(n){return this.#private(n)}}
            let c=new C;
            function forwarded(n){if(n===0)return 9;return proxy.call(null,n-1)}
            let proxy=new Proxy(forwarded.bind(null),{});
            even(512)&&!odd(512)&&arrow(512)===7&&c.m(512)===c&&c.private(512)===c&&forwarded(512)===9
        "#,
            )
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn tail_expressions_preserve_short_circuiting_receivers_and_argument_order() {
    check(
        r#"
        'use strict';let log=[];
        function f(n){return n===0?17:(true&&true&&f(n-1))}
        function g(n){return n===0?19:(false||false||g(n-1))}
        function h(n){return n===0?23:(undefined??null??h(n-1))}
        function k(n){return n===0?29:(0,1,k(n-1))}
        let o={get m(){log.push('get');return function(a,b){if(this!==o)throw 1;log.push('call');return a+b}}};
        function ordered(){return o.m((log.push('a'),3),...(log.push('b'),[4]))}
        function direct(){let local=31;return eval('local')}
        function skipped(){return 0&&missing()}
        f(256)===17&&g(256)===19&&h(256)===23&&k(256)===29&&
            ordered()===7&&log.join(',')==='get,a,b,call'&&direct()===31&&skipped()===0
    "#,
    );
}

#[test]
fn handlers_and_finalizers_observe_actual_call_results_before_tail_transfers() {
    check(
        r#"
        'use strict';let log=[];
        function fail(){log.push('call');throw 7}
        function loop(n){if(n===0)return 11;return loop(n-1)}
        function caught(){try{return fail()}catch(e){log.push(e);return loop(256)}}
        function finalized(){try{return fail()}finally{log.push('finally')}}
        function override(){try{throw 9}finally{return loop(256)}}
        let a=caught(),b;try{finalized()}catch(e){b=e}
        a===11&&b===7&&override()===11&&log.join(',')==='call,7,call,finally'
    "#,
    );
}

#[test]
fn iterator_closing_and_lexical_disposal_follow_the_called_function() {
    check(
        r#"
        'use strict';let log=[];
        function called(){log.push('call');return 7}
        let iterable={[Symbol.iterator](){return {next(){return {value:1,done:false}},return(){log.push('close');return {}}}}};
        function iterated(){for(let v of iterable)return called()}
        function disposed(){using r={[Symbol.dispose](){log.push('dispose')}};return called()}
        function deep(n){if(n===0)return 3;return deep(n-1)}
        function caller(){using r={[Symbol.dispose](){log.push('outer')}};return deep(256)}
        let normal=iterated()===7&&disposed()===7&&caller()===3&&log.join(',')==='call,close,call,dispose,outer';
        function fail(){log.push('fail');throw 1}
        function abrupt(){for(let v of iterable)return fail()}
        function suppressed(){using r={[Symbol.dispose](){log.push('suppress');throw 2}};return fail()}
        let a,b;try{abrupt()}catch(e){a=e}try{suppressed()}catch(e){b=e}
        normal&&a===1&&b instanceof SuppressedError&&b.error===2&&b.suppressed===1&&log.slice(-4).join(',')==='fail,close,fail,suppress'
    "#,
    );
}

#[test]
fn returned_closures_and_constructor_results_survive_caller_retirement_and_gc() {
    let mut realm = Realm::default();
    assert_eq!(
        realm.eval(
            r#"
        'use strict';
        function capture(x){let object={x};return (()=>()=>object)()}
        function empty(n){if(n!==0)return empty(n-1)}
        class C{constructor(){this.x=13;return empty(256)}}
        class D extends C{constructor(){super();return empty(256)}}
        let saved=capture(17),c=new C,d=new D;
        c.x===13&&d.x===13
    "#
        ),
        Ok(Value::Boolean(true))
    );
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("saved().x===17&&c.x===13&&d.x===13"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn ordinary_recursion_keeps_its_guard_and_tail_loops_keep_opted_in_work_limits() {
    for (index, source) in [
        "function f(){return f()}f()",
        "function f(){'use strict';return 1+f()}f()",
        "function f(){'use strict';try{return f()}catch(e){return 1}}f()",
        "function f(){'use strict';for(let x of [1])return f()}f()",
        "async function f(){'use strict';return f()}f()",
        "function* f(){'use strict';return f().next()}f().next()",
    ]
    .into_iter()
    .enumerate()
    {
        std::thread::Builder::new()
            .name(format!("tail-guard-{index}"))
            .stack_size(2 * 1024 * 1024)
            .spawn(move || {
                assert!(
                    matches!(Realm::default().eval(source), Err(Error::Limit { .. })),
                    "{source}"
                )
            })
            .unwrap()
            .join()
            .unwrap();
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(20000),
        ..Limits::default()
    });
    assert!(matches!(realm.eval("let flag=0;function f(){'use strict';return f()}try{f()}catch(e){flag=1}finally{flag=2}"),Err(Error::Limit{..})));
    assert_eq!(
        realm.eval("flag===0&&(()=>{'use strict';return 7})()===7"),
        Ok(Value::Boolean(true))
    );
}
