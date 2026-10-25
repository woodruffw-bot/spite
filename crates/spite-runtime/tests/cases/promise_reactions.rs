//! Promise reaction scheduling, species and traced captures (27.2.5.4).

use spite_runtime::{Error, PromiseJobOutcome, Realm, Value};

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
fn pending_reactions_run_asynchronously_in_registration_order_and_append_children() {
    let mut realm = Realm::default();
    realm.eval("let log='',resolve,p=new Promise(r=>{resolve=r}),a=p.then(function(v){'use strict';if(this!==undefined||arguments.length!==1||v!==7)throw 1;log+='a';return v+1});p.then(v=>{log+='b';return v});a.then(v=>{if(v!==8)throw 2;log+='c'});resolve(7);").unwrap();
    assert_eq!(realm.eval("log"), Ok(Value::String("".into())));
    for expected in ["a", "ab", "abc"] {
        realm.collect(usize::MAX).unwrap();
        assert!(matches!(
            realm.run_next_promise_job(),
            Ok(PromiseJobOutcome::Completed)
        ));
        assert_eq!(realm.eval("log"), Ok(Value::String(expected.into())));
    }
    assert!(matches!(
        realm.run_next_promise_job(),
        Ok(PromiseJobOutcome::NoJob)
    ));
}

#[test]
fn absent_handlers_propagate_values_and_reasons_and_catch_recovers() {
    let mut realm = Realm::default();
    realm.eval("let seen=[],marker={};Promise.reject(marker).then(null,false).catch(e=>{if(e!==marker)throw 1;seen.push('r');return 3}).then(v=>{seen.push(v)});Promise.resolve(9).then(1,2).then(v=>{seen.push(v)});").unwrap();
    drain(&mut realm);
    assert_eq!(
        realm.eval("seen.join(',')"),
        Ok(Value::String("r,9,3".into()))
    );
    realm.eval("let caught=false;Promise.resolve(1).then(class C{}).catch(e=>{caught=e instanceof TypeError});").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("caught"), Ok(Value::Boolean(true)));
}

#[test]
fn branding_precedes_hooks_and_species_construction_can_settle_the_parent() {
    check(
        "let then=Promise.prototype.then,traps=0,fake=Object.create(Promise.prototype),proxy=new Proxy(Promise.resolve(1),{get(){traps++;throw 1}});Object.defineProperty(fake,'constructor',{get(){traps++;throw 2}});let caught=0;for(const value of [undefined,null,1,{},fake,proxy]){try{then.call(value)}catch(e){if(e instanceof TypeError)caught++}}caught===6&&traps===0",
    );
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let log='',resolve,p=new Promise(r=>{resolve=r}),result={};function C(executor){log+='c';resolve(7);executor(function(v){'use strict';if(this!==undefined||arguments.length!==1||v!==8)throw 1;log+='r8'},()=>{throw 2});return result}p.constructor={get [Symbol.species](){log+='s';return C}};let out=p.then(v=>{log+='h';return v+1});out===result&&log==='sc'"),Ok(Value::Boolean(true)));
    drain(&mut realm);
    assert_eq!(realm.eval("log"), Ok(Value::String("schr8".into())));
}

#[test]
fn child_self_resolution_rejects_and_custom_resolver_throws_are_reported() {
    let mut realm = Realm::default();
    realm.eval("let child,caught=false;child=Promise.resolve(1).then(()=>child);child.catch(e=>{caught=e instanceof TypeError});").unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("caught"), Ok(Value::Boolean(true)));
    realm.eval("let marker={},rejected=false,p=Promise.resolve(1);function C(executor){executor(()=>{throw marker},()=>{rejected=true})}p.constructor={[Symbol.species]:C};p.then(()=>2);").unwrap();
    let PromiseJobOutcome::Reported(Error::Thrown(value)) = realm.run_next_promise_job().unwrap()
    else {
        panic!("reported resolver throw")
    };
    assert_eq!(realm.eval("marker"), Ok(value));
    assert_eq!(realm.eval("rejected"), Ok(Value::Boolean(false)));
}

#[test]
fn pending_and_queued_reaction_captures_survive_collection_and_release_after_execution() {
    let mut realm = Realm::default();
    realm.eval("let calls=0,state=(()=>{let value={},symbol=Symbol(),resolve,p=new Promise(r=>{resolve=r});p.then(()=>{calls++;return[value,symbol]});return{p,resolve,object:new WeakRef(value),symbol:new WeakRef(symbol)}})();").unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("state.object.deref()!==undefined&&state.symbol.deref()!==undefined"),
        Ok(Value::Boolean(true))
    );
    realm
        .eval("state.resolve(1);state.p=null;state.resolve=null")
        .unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("calls===1&&state.object.deref()===undefined&&state.symbol.deref()===undefined"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn default_realms_accept_large_reaction_lists_without_automatic_execution() {
    let mut realm = Realm::default();
    assert_eq!(
        realm.eval(
            "let calls=0,p=Promise.resolve(1);for(let i=0;i<5000;i++)p.then(()=>{calls++});calls"
        ),
        Ok(Value::Number(0.0))
    );
    drain(&mut realm);
    assert_eq!(realm.eval("calls"), Ok(Value::Number(5000.0)));
}
