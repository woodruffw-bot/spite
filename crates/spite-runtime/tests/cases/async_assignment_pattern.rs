//! Destructuring assignment suspension, saved references and iterator closing.

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
fn array_targets_precede_steps_defaults_and_final_raw_key_conversion() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],out={},result;
        let key={toString(){events.push('convert');return 'x'}};
        let source={[Symbol.iterator](){events.push('iter');return this},
            get next(){events.push('next-get');return function(){events.push('next');return {done:false,value:undefined}}},
            return(){events.push('return');return {}}};
        async function f(){return ([out[(events.push('target'),await key)]=(events.push('default'),await 7)]=source)===source}
        f().then(v=>result=v);
    "#).unwrap();
    check(&mut realm, "events.join(',')==='iter,next-get,target'");
    drain(&mut realm);
    check(
        &mut realm,
        "result&&out.x===7&&events.join(',')==='iter,next-get,target,next,default,convert,return'",
    );
}

#[test]
fn object_keys_targets_getters_defaults_and_rest_keep_their_order() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result;
        let key={[Symbol.toPrimitive](){events.push('key');return 's'}};
        let out={set x(value){events.push('set:'+value)}};
        let source=new Proxy({get s(){events.push('get:s');return undefined},get y(){events.push('get:y');return 8}},
            {ownKeys(t){events.push('keys');return Reflect.ownKeys(t)}});
        async function f(){return ({[await key]:out[(events.push('target'),await 'x')]=(events.push('default'),await 7),
            ...out[(events.push('rest-target'),await 'rest')]}=source)===source}
        f().then(v=>result=v);
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result&&out.rest.y===8&&Object.keys(out.rest).join(',')==='y'&&events.join(',')==='key,target,get:s,default,set:7,rest-target,keys,get:y'",
    );
}

#[test]
fn nested_assignments_and_patterns_preserve_rhs_identity_and_default_class_names() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let result;
        async function f(){
            let A,B,out={},x,y,rhs=[];
            ([A=class{[await 'm'](){}static seen=this.name}]=[]);
            ({a:B=(class extends(await Object){static seen=this.name})}={});
            ({a:out.x=class{[await 'm'](){}static seen=this.name}}={});
            let identity=([[x=await 1]=([y=await 2]=[])]=rhs)===rhs;
            return [A.seen,B.seen,out.x.seen,x,y,identity];
        }
        f().then(v=>result=v);
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "result.join(',')==='A,B,,1,2,true'");
}

#[test]
fn exhausted_arrays_still_evaluate_targets_and_rest_can_receive_nested_patterns() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],out={},result;
        let source={[Symbol.iterator](){events.push('iter');return this},next(){events.push('next');return {done:true}},return(){throw 1}};
        async function f(){let z;
            [,out[(events.push('a'),await 'a')]=await 1,out[(events.push('b'),await 'b')]=await 2,...out[(events.push('rest'),await 'rest')]]=source;
            ([...[z=await 3]]=[]);
            return z;
        }
        f().then(v=>result=v);
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "result===3&&out.a===1&&out.b===2&&out.rest.length===0&&events.join(',')==='iter,next,a,b,rest'",
    );
}

#[test]
fn rejection_closes_inner_iterators_before_outer_and_preserves_the_original_throw() {
    for pattern in [
        "([[x=await Promise.reject(marker)]]=make('outer',make('inner',undefined)))",
        "([x=class C{[([y=await Promise.reject(marker)]=make('inner',undefined))](){}}]=make('outer',undefined))",
        "let C=class Outer{[([[x=class Inner{[await Promise.reject(marker)](){}}]]=make('outer',make('inner',undefined)))](){}}",
    ] {
        let mut realm = Realm::default();
        realm.eval("let events=[],marker={},x,y;function make(tag,value){return {[Symbol.iterator](){return this},next(){return {done:false,value}},return(){events.push(tag);throw tag}}}").unwrap();
        realm.eval(&format!("async function f(){{try{{{pattern}}}catch(e){{events.push(e===marker)}}finally{{events.push('finally')}}}}f()")).unwrap();
        drain(&mut realm);
        check(&mut realm, "events.join(',')==='inner,outer,true,finally'");
    }
}

#[test]
fn generator_return_closes_patterns_and_close_errors_override_return_completions() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let events=[],closeError={},x;
        let source={[Symbol.iterator](){return this},next(){return {done:false}},return(){events.push('close');throw closeError}};
        function* g(){try{[x=yield 'pause']=source}catch(e){events.push(e===closeError);return 77}finally{patternSloppy=4;events.push('finally')}}
        let iter=g();if(iter.next().value!=='pause')throw 1;let done=iter.return(99);
        done.done&&done.value===77&&patternSloppy===4&&events.join(',')==='close,true,finally'
    "#,
    );
}

#[test]
fn saved_raw_target_references_keep_objects_and_symbols_alive_only_while_suspended() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,weakTarget,weakKey,result;let gate=new Promise(r=>release=r);
        async function f(){let target={},key=Symbol();weakTarget=new WeakRef(target);weakKey=new WeakRef(key);
            [target[key]=(target=null,key=null,await gate)]=[undefined];
            return weakTarget.deref()[weakKey.deref()]===7;
        }
        f().then(v=>result=v);
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "weakTarget.deref()!==undefined&&weakKey.deref()!==undefined",
    );
    realm.eval("release(7)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true");
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "weakTarget.deref()===undefined&&weakKey.deref()===undefined",
    );
}

#[test]
fn excluded_symbol_keys_survive_later_defaults_and_are_released_after_rest_copy() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,weakKey,result;let gate=new Promise(r=>release=r);
        async function f(){let key=Symbol(),picked,x,rest;weakKey=new WeakRef(key);
            let source=new Proxy({}, {ownKeys(){return [weakKey.deref(),'y']},
                getOwnPropertyDescriptor(){return {enumerable:true,configurable:true}},get(t,k){return k==='x'?undefined:1}});
            ({[key]:picked,x=(key=null,await gate),...rest}=source);
            return picked===1&&x===7&&rest.y===1&&Object.getOwnPropertySymbols(rest).length===0;
        }
        f().then(v=>result=v);
    "#).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "weakKey.deref()!==undefined");
    realm.eval("release(7)").unwrap();
    drain(&mut realm);
    check(&mut realm, "result===true");
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "weakKey.deref()===undefined");
}

#[test]
fn yielded_keys_targets_and_defaults_share_the_value_expression_task_stack() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let out={},rest,source={p:undefined,q:9};
        function* g(){return ({[yield 'key']:out[yield 'target']=yield 'default',...out[yield 'rest-target']}=source)}
        let iter=g();if(iter.next().value!=='key'||iter.next('p').value!=='target'||iter.next('x').value!=='default'||iter.next(7).value!=='rest-target')throw 1;
        let done=iter.next('rest');
        done.done&&done.value===source&&out.x===7&&out.rest.q===9&&!Object.hasOwn(out.rest,'p')
    "#,
    );
}

#[test]
fn host_abort_drops_assignment_captures_without_iterator_or_language_cleanup() {
    for pattern in [
        "[target[key]=(target=null,key=null,await 0,import('unavailable'))]=source",
        "[target[(key=null,await 0,import('unavailable'))]]=source",
    ] {
        let mut realm = Realm::default();
        realm.eval("let flag=0,closes=0,weak;let source={[Symbol.iterator](){return this},next(){return {done:false}},return(){closes++;return {}}}").unwrap();
        realm.eval(&format!("async function f(){{let target={{}},key=Symbol();weak=new WeakRef(target);try{{{pattern}}}catch{{flag=1}}finally{{flag=2}}}}let pending=f()")).unwrap();
        assert!(matches!(
            realm.run_next_promise_job(),
            Ok(PromiseJobOutcome::Reported(Error::Unsupported { .. }))
        ));
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        check(
            &mut realm,
            "flag===0&&closes===0&&weak.deref()===undefined&&(assignmentEmbeddingSloppy=7)===7",
        );
    }
}

#[test]
fn async_generator_targets_and_defaults_preserve_request_queue_order() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let events=[];
        async function* g(){let out={},rhs=[undefined];
            let value=([out[yield 'key']=await Promise.resolve(7)]=rhs);
            yield [value===rhs,out.x];return 9;
        }
        let iter=g();
        iter.next().then(r=>events.push(r.value));
        iter.next('x').then(r=>events.push(r.value.join(':')));
        iter.next().then(r=>events.push(r.value));
    "#,
        )
        .unwrap();
    drain(&mut realm);
    check(&mut realm, "events.join(',')==='key,true:7,9'");
}
