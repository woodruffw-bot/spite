//! Generator function objects, lazy native bodies and initial/completed resumes.

use spite_runtime::{Error, Limits, PromiseJobOutcome, Realm, Value};

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}
fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn constructor_function_and_generator_prototypes_have_standard_metadata() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        function*g(x,y=1){}let GF=g.constructor,FP=Object.getPrototypeOf(g),GP=FP.prototype;
        let p=Object.getOwnPropertyDescriptor(g,'prototype'),c=Object.getOwnPropertyDescriptor(GP,'constructor');
        typeof GeneratorFunction==='undefined'&&typeof GF==='function'&&typeof FP==='object'&&GF.name==='GeneratorFunction'&&GF.length===1&&g.length===1&&g.name==='g'&&
        Object.getPrototypeOf(GF)===Function&&GF.prototype===FP&&FP.constructor===GF&&Object.getPrototypeOf(FP)===Function.prototype&&Object.getPrototypeOf(GP)===Iterator.prototype&&
        Object.getPrototypeOf(g.prototype)===GP&&!Object.hasOwn(g.prototype,'constructor')&&g.prototype.constructor===FP&&p.writable&&!p.enumerable&&!p.configurable&&!c.writable&&!c.enumerable&&c.configurable&&
        Reflect.ownKeys(FP).map(key=>typeof key==='symbol'?key.toString():key).join(',')==='constructor,prototype,Symbol(Symbol.toStringTag)'&&Object.prototype.toString.call(g)==='[object GeneratorFunction]'&&Object.prototype.toString.call(g())==='[object Generator]'
    "#,
    );
}

#[test]
fn parameters_initialize_at_call_and_bodies_wait_until_first_next() {
    let mut realm = Realm::default();
    realm.eval("let events=[];function*g(x=(events.push('parameter'),7)){events.push('body');return x}let iterator=g();").unwrap();
    check(
        &mut realm,
        "events.join(',')==='parameter'&&iterator[Symbol.iterator]()===iterator&&iterator instanceof Iterator&&iterator instanceof g",
    );
    check(
        &mut realm,
        "let first=iterator.next(99),second=iterator.next(100);first.done&&first.value===7&&second.done&&second.value===undefined&&events.join(',')==='parameter,body'",
    );
    check(&mut realm, "(function*(){7})().next().value===undefined");
}

#[test]
fn parameter_errors_are_synchronous_and_close_binding_iterators_before_creation() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let reason={},events=[],body=0,result=false;function fail(){events.push('default');throw reason}
        let source={[Symbol.iterator](){return this},next(){events.push('next');return{done:false,value:undefined}},return(){events.push('close');return{}}};
        function*g([x=fail()]){body++}try{g(source)}catch(error){result=error===reason}
        let tdz=false;function*h(x=x){}try{h()}catch(error){tdz=error instanceof ReferenceError}
        result&&tdz&&body===0&&events.join(',')==='next,default,close'
    "#,
    );
}

#[test]
fn mapped_arguments_this_and_new_target_keep_the_original_call_context() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        function*a(x){x=7;return arguments[0]}function*b(x){'use strict';x=9;return arguments[0]}function*c(x=3){x=11;return arguments[0]}
        function*d(){return [typeof this,this.valueOf(),new.target===undefined]}
        a(1).next().value===7&&b(1).next().value===1&&c(1).next().value===1&&d.call(5).next().value.join(',')==='object,5,true'&&
        (function*(){'use strict';return this}).call(5).next().value===5
    "#,
    );
}

#[test]
fn return_and_throw_before_start_skip_the_body_and_completed_resumes_keep_identity() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let body=0,finalized=0,reason={};function*g(){body++;try{return 7}finally{finalized++}}
        let a=g(),returned=a.return(reason),b=g(),thrown=false;try{b.throw(reason)}catch(error){thrown=error===reason}
        let c=g(),first=c.next(),later=c.return(reason),again=false;try{c.throw(reason)}catch(error){again=error===reason}
        returned.done&&returned.value===reason&&thrown&&a.next().done&&b.next().done&&first.done&&first.value===7&&later.done&&later.value===reason&&again&&c.next().value===undefined&&body===1&&finalized===1
    "#,
    );
}

#[test]
fn prototype_methods_reject_incompatible_receivers_and_all_reentrant_resumes() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let GP=Object.getPrototypeOf(function*(){}).prototype,count=0;
        for(const name of ['next','return','throw'])for(const value of [undefined,null,0,{},GP,[],new Proxy((function*(){})(),{})]){try{GP[name].call(value)}catch(error){if(error instanceof TypeError)count++}}
        let current,seen=[];function*g(){for(const name of ['next','return','throw']){try{GP[name].call(current,1)}catch(error){seen.push(error instanceof TypeError)}}return 7}
        current=g();let result=current.next();count===21&&result.done&&result.value===7&&seen.length===3&&seen.every(x=>x===true)
    "#,
    );
}

#[test]
fn generator_methods_keep_live_home_objects_private_brands_and_receivers() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let object={__proto__:{x:7},*g(){return super.x+this.y}},iterator=object.g.call({y:2});Object.setPrototypeOf(object,{x:11});
        class C{#x=17;*g(){return this.#x}static marker=19;static *h(){return this.marker}}
        let c=new C,privateIterator=c.g(),method=object.g,p=Object.getOwnPropertyDescriptor(method,'prototype'),notConstructor=false;
        try{new method()}catch(error){notConstructor=error instanceof TypeError}
        iterator.next().value===13&&privateIterator.next().value===17&&C.h().next().value===19&&p.writable&&!p.enumerable&&!p.configurable&&notConstructor
    "#,
    );
}

#[test]
fn dynamic_construction_uses_global_scope_standard_source_and_ordered_new_target_lookup() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let GF=(function*(){}).constructor,events=[],custom={},target=new Proxy(function(){},{get(object,key){if(key==='prototype'){events.push('prototype');return custom}return Reflect.get(object,key)}});
        let f=Reflect.construct(GF,[{toString(){events.push('parameters');return 'x'}},{toString(){events.push('body');return 'return x'}}],target);
        let notConstructor=false;try{new f()}catch(error){notConstructor=error instanceof TypeError}
        function local(){let hidden=7;return GF('return typeof hidden')}
        Object.getPrototypeOf(f)===custom&&f(23).next().value===23&&events.join(',')==='parameters,body,prototype'&&notConstructor&&
        GF('return 13')().next().value===13&&(new GF('return 17'))().next().value===17&&local()().next().value==='undefined'&&f.toString===Object.prototype.toString&&
        GF('yield 1').toString()==='function* anonymous(\n) {\nyield 1\n}'
    "#,
    );
}

#[test]
fn native_body_completions_run_finalizers_and_resource_disposal_once() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let reason={},events=[],resource={[Symbol.dispose](){events.push('dispose')}};
        function*g(){using r=resource;try{return 7}finally{events.push('finally')}}let iterator=g();let before=events.length;let result=iterator.next();iterator.return(9);
        function*h(){using r=resource;try{throw reason}finally{events.push('throw-finally')}}let other=h(),thrown=false;try{other.next()}catch(error){thrown=error===reason}
        before===0&&result.done&&result.value===7&&thrown&&other.next().done&&events.join(',')==='finally,dispose,throw-finally,dispose'
    "#,
    );
}

#[test]
fn host_aborts_complete_generators_release_captures_and_skip_javascript_cleanup() {
    let mut resumed = Realm::default();
    resumed.eval("let before=0,operand=0,disposed=0,finalized=0,caught=0;let resource={[Symbol.dispose](){disposed++}},weak=new WeakRef(resource[Symbol.dispose]);function*g(){using r=resource;delete resource[Symbol.dispose];try{before++;yield(operand++)}catch{caught++}finally{finalized++}}let iterator=g();iterator.next()").unwrap();
    collect(&mut resumed);
    check(
        &mut resumed,
        "before===1&&operand===1&&disposed===0&&finalized===0&&caught===0&&weak.deref()!==undefined",
    );
    check(
        &mut resumed,
        "iterator.next().done&&disposed===1&&finalized===1&&caught===0",
    );
    collect(&mut resumed);
    check(&mut resumed, "weak.deref()===undefined");
    let mut delegated = Realm::default();
    delegated.eval("let before=0,operand=0,disposed=0,finalized=0,caught=0;let resource={[Symbol.dispose](){disposed++}},weak=new WeakRef(resource[Symbol.dispose]);function*g(){using r=resource;delete resource[Symbol.dispose];try{before++;yield*(operand++)}catch(error){if(error instanceof TypeError)caught++}finally{finalized++}}let iterator=g();iterator.next()").unwrap();
    collect(&mut delegated);
    check(
        &mut delegated,
        "before===1&&operand===1&&disposed===1&&finalized===1&&caught===1&&iterator.next().done&&weak.deref()===undefined",
    );
    for limited in [false, true] {
        let mut realm = Realm::new(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let before=0,operand=0,disposed=0,finalized=0,caught=0;function spin(){while(true){}}").unwrap();
        realm.eval("let resource={[Symbol.dispose](){disposed++}},weak=new WeakRef(resource[Symbol.dispose]);").unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "Function('async function* gap(){}')()"
        };
        realm.eval(&format!("function*g(){{using r=resource;delete resource[Symbol.dispose];try{{before++;{tail}}}catch{{caught++}}finally{{finalized++}}}}let iterator=g()")).unwrap();
        let error = realm.eval("iterator.next()").unwrap_err();
        if limited {
            assert!(matches!(error, Error::Limit { .. }), "{error}");
        } else {
            assert!(matches!(error, Error::Unsupported { .. }), "{error}");
        }
        collect(&mut realm);
        check(
            &mut realm,
            "before===1&&operand===0&&disposed===0&&finalized===0&&caught===0&&iterator.next().done&&weak.deref()===undefined",
        );
    }
}

#[test]
fn suspended_start_retains_the_callee_and_environment_then_releases_them_on_completion() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let weak,calleeWeak,iterator=(()=>{let kept={};weak=new WeakRef(kept);let f=function*(arguments){kept;return arguments};calleeWeak=new WeakRef(f);let result=f(7);f=null;return result})();
    "#).unwrap();
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
fn synchronous_and_async_consumers_observe_completion_and_early_return_without_body_replay() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let count=0,result;function*g(){count++;return 7}let skipped=g();let values=Iterator.from(skipped).take(0).toArray();
        let direct=[...g()],from=Array.from(g());async function f(){let seen=0;for await(const x of g())seen++;return seen}f().then(value=>{result=value});
    "#).unwrap();
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
    check(
        &mut realm,
        "values.length===0&&skipped.next().done&&direct.length===0&&from.length===0&&result===0&&count===3",
    );
}
