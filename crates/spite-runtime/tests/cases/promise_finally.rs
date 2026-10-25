//! Generic finally dispatch, cleanup assimilation and native captures (27.2.5.3).

use spite_runtime::{PromiseJobOutcome, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
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
fn generic_dispatch_observes_species_before_then_and_preserves_noncallable_arguments() {
    check(
        "let f=Promise.prototype.finally,log='',marker={},x={get constructor(){log+='c';return{get [Symbol.species](){log+='s';return null}}},get then(){log+='t';return function(a,b){'use strict';return this===x&&arguments.length===2&&a===marker&&b===marker}}};f.call(x,marker)&&log==='cst'",
    );
    check(
        "let f=Promise.prototype.finally,caught=0;for(const x of [undefined,null,0,'',true,Symbol(),1n]){try{f.call(x)}catch(e){if(e instanceof TypeError)caught++}}caught===7",
    );
    check(
        "let marker={},called=false,x={get constructor(){throw marker},get then(){called=true}};try{Promise.prototype.finally.call(x)}catch(e){if(e!==marker)throw 1}!called",
    );
}

#[test]
fn cleanup_preserves_settlement_and_waits_for_thenables_with_zero_callback_arguments() {
    let mut realm = Realm::default();
    realm.eval("let seen=[],finish,marker={},original=Promise.resolve(marker);original.finally(function(){'use strict';if(this!==undefined||arguments.length!==0)throw 1;seen.push('cleanup');return{then(resolve){finish=resolve}}}).then(v=>{if(v!==marker)throw 2;seen.push('value')});Promise.reject(marker).finally(()=>7).catch(e=>{if(e!==marker)throw 3;seen.push('reason')});").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("seen.join(',')"),
        Ok(Value::String("cleanup,reason".into()))
    );
    realm.eval("finish(42)").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("seen.join(',')"),
        Ok(Value::String("cleanup,reason,value".into()))
    );
    realm.eval("let replacement={},count=0;for(const p of [Promise.resolve(1),Promise.reject(2)]){p.finally(()=>{throw replacement}).catch(e=>{if(e!==replacement)throw 4;count++});p.finally(()=>Promise.reject(replacement)).catch(e=>{if(e!==replacement)throw 5;count++})}").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("count"), Ok(Value::Number(4.0)));
}

#[test]
fn wrappers_capture_species_and_invoke_cleanup_then_with_one_native_continuation() {
    check(
        "let log='',handlers,result={},reason=Symbol(),continuation;function C(executor){log+='c';executor(function(v){'use strict';if(this!==undefined||arguments.length!==1||v!==9)throw 1;log+='r'},()=>{});return{then(f){if(arguments.length!==1)throw 2;continuation=f;return result}}}let x={constructor:{[Symbol.species]:C},then(a,b){handlers=[a,b];return result}},callback=function(){'use strict';if(this!==undefined||arguments.length)throw 3;log+='f';return 9};let returned=Promise.prototype.finally.call(x,callback);x.constructor=null;let value=handlers[0](reason);let a=continuation;let rejected=handlers[1](reason);let b=continuation,caught=false;try{b()}catch(e){caught=e===reason}returned===result&&value===result&&rejected===result&&a()===reason&&a.length===0&&a.name===''&&b.length===0&&b.name===''&&log==='fcrfcr'&&caught",
    );
}

#[test]
fn native_wrapper_and_continuation_captures_survive_collection_and_release() {
    let mut realm = Realm::default();
    realm.eval("let handler,continuation;let state=(()=>{let callback=Function(''),ctor=Function('executor','executor(()=>{},()=>{});return{then(fn){continuation=fn}}'),x={constructor:{[Symbol.species]:ctor},then(fn){handler=fn}};Promise.prototype.finally.call(x,callback);return{callback:new WeakRef(callback),ctor:new WeakRef(ctor)}})();").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.callback.deref()!==undefined&&state.ctor.deref()!==undefined"),
        Ok(Value::Boolean(true))
    );
    realm.eval("let refs=(()=>{let value={},symbol=Symbol();handler([value,symbol]);return{value:new WeakRef(value),symbol:new WeakRef(symbol)}})();handler=null").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("continuation()[0]===refs.value.deref()&&continuation()[1]===refs.symbol.deref()&&state.callback.deref()===undefined&&state.ctor.deref()===undefined"), Ok(Value::Boolean(true)));
    realm.eval("continuation=null").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("refs.value.deref()===undefined&&refs.symbol.deref()===undefined"),
        Ok(Value::Boolean(true))
    );
}
