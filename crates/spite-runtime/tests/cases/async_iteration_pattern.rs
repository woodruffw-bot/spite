//! Assignment-pattern iteration heads share the suspended assignment engine.

use spite_runtime::{Error, PromiseJobOutcome, Realm, Value};

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
fn for_of_patterns_pause_before_body_and_assign_each_iteration_value() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let events=[],result;
        async function f(){let x,out={},sum=0;
            for([out[await 'x']=await 2] of [[undefined],[3]]){events.push(out.x);sum+=out.x}
            for({[await 'p']:x=await 4} of [{},{p:5}]){events.push(x);sum+=x}
            return sum;
        }
        f().then(v=>result=v);
    "#,
        )
        .unwrap();
    check(&mut realm, "events.length===0");
    drain(&mut realm);
    check(&mut realm, "result===14&&events.join(',')==='2,3,4,5'");
}

#[test]
fn for_in_patterns_destructure_key_strings_before_body_entry() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let events=[];
        async function f(){let x,y;
            for([x=await 7] in {'':1})events.push(x);
            for({[await 'length']:y} in {abc:1})events.push(y);
        }
        f();
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='7,3'");
}

#[test]
fn for_await_patterns_accept_sync_and_async_iterator_values() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[];
        async function f(){let x;
            for await([x=await 2] of [[undefined],[3]])events.push(x);
            let i=0,values=[{},{p:4}],source={[Symbol.asyncIterator](){return this},next(){return Promise.resolve(i<values.length?{done:false,value:values[i++]}:{done:true})}};
            for await({[await 'p']:x=await 5} of source)events.push(x);
        }
        f();
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='2,3,5,4'");
}

#[test]
fn rejected_head_defaults_close_inner_patterns_before_sync_or_async_outer_iterators() {
    for asynchronous in [false, true] {
        let mut realm = Realm::default();
        realm
            .eval(&format!(
                "let asynchronous={asynchronous},events=[],marker={{}},x;"
            ))
            .unwrap();
        realm.eval(r#"
            let inner={[Symbol.iterator](){return this},next(){return {done:false}},return(){events.push('inner');return {}}};
            let outer={[Symbol.iterator](){return this},[Symbol.asyncIterator](){return this},
                next(){let value={done:false,value:inner};return asynchronous?Promise.resolve(value):value},
                return(){events.push('outer');return asynchronous?Promise.resolve().then(()=>{events.push('outer-done');return {}}):{}}};
            async function f(){try{
                if(asynchronous){for await([x=await Promise.reject(marker)] of outer)events.push('wrong')}
                else{for([x=await Promise.reject(marker)] of outer)events.push('wrong')}
            }catch(e){events.push(e===marker)}finally{events.push('finally')}}
            f();
        "#).unwrap();
        drain(&mut realm);
        check(
            &mut realm,
            if asynchronous {
                "events.join(',')==='inner,outer,outer-done,true,finally'"
            } else {
                "events.join(',')==='inner,outer,true,finally'"
            },
        );
    }
}

#[test]
fn yielded_assignment_heads_preserve_labelled_continue_and_break() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let events=[];
        function* g(){let x;outer:for([x=yield 'head'] of [[undefined],[undefined]]){events.push(x);if(x===1)continue outer;break outer}return 9}
        let iter=g();if(iter.next().value!=='head'||iter.next(1).value!=='head')throw 1;
        let done=iter.next(2);done.done&&done.value===9&&events.join(',')==='1,2'
    "#,
    );
}

#[test]
fn pending_head_references_are_traced_and_body_waits_for_initialization() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,weakTarget,weakKey,result,body=0;let gate=new Promise(r=>release=r);
        async function f(){let target={},key=Symbol();weakTarget=new WeakRef(target);weakKey=new WeakRef(key);
            for([target[key]=(target=null,key=null,await gate)] of [[undefined]]){body++;return weakTarget.deref()[weakKey.deref()]===7}
        }
        f().then(v=>result=v);
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "body===0&&weakTarget.deref()!==undefined&&weakKey.deref()!==undefined",
    );
    realm.eval("release(7)").unwrap();
    drain(&mut realm);
    check(&mut realm, "body===1&&result===true");
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "weakTarget.deref()===undefined&&weakKey.deref()===undefined",
    );
}

#[test]
fn nested_assignment_defaults_and_class_names_use_iteration_pattern_paths() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let events=[];
        async function f(){let A,x,y;
            for([A=class{[await 'm'](){}static seen=this.name}] of [[]])events.push(A.seen);
            for({[await 'p']:[x=([y=await 3]=[],await 4)]} of [{p:[]}])events.push(x,y);
        }
        f();
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='A,4,3'");
}

#[test]
fn head_host_abort_skips_outer_close_body_rejection_and_finally() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let flag=0,body=0,closed=0,x;
        let source={[Symbol.asyncIterator](){return this},next(){return Promise.resolve({done:false,value:[]})},return(){closed++;return {}}};
        async function f(){try{for await([x=(await 0,import('unavailable'))] of source)body++}catch{flag=1}finally{flag=2}}
        f().catch(()=>flag=3);
    "#).unwrap();
    let mut reported = false;
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => {
                assert!(matches!(error, Error::Unsupported { .. }));
                reported = true;
            }
        }
    }
    assert!(reported);
    check(&mut realm, "flag===0&&body===0&&closed===0");
}
