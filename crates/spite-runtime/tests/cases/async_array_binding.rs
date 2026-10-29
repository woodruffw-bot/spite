//! Lexical Array BindingInitialization with suspended defaults (14.3.3, Await).

use crate::common::UnavailableHostRealm;
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
fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn defaults_resume_in_order_before_later_elements_and_declarators() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result;async function f(){let [x=await(events.push('x'),7),y=await(events.push(x),9)]=[],tail=(events.push('tail'),11);return [x,y,tail].join(',')}f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "events.join(',')==='x'&&result===undefined");
    drain(&mut realm);
    check(
        &mut realm,
        "events.join(',')==='x,7,tail'&&result==='7,9,11'",
    );
}

#[test]
fn defined_values_and_unselected_await_branches_skip_their_operands() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let effects=0,result;async function f(){let [x=await(effects++,7),y=false?await missing:()=>{}]=[1,undefined];return x===1&&typeof y==='function'&&y.name===''&&effects===0}f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "effects===0");
    drain(&mut realm);
    check(&mut realm, "result===true&&effects===0");
}

#[test]
fn nested_arrays_rest_patterns_and_elisions_keep_iterator_protocol_order() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,reads=0,calls=0;let source={[Symbol.iterator](){return this},next(){calls++;if(calls===1)return{done:false,get value(){reads++;throw 1}};return calls===2?{done:false,value:[]}:calls===3?{done:false,value:undefined}:calls===4?{done:false,value:11}:calls===5?{done:false,value:12}:{done:true}}};
        async function f(){let [,[x=await 7],...[y=await 9,...tail]]=source;return [x,y,tail.join(',')].join(';')}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result==='7;9;11,12'&&reads===0&&calls===6");
}

#[test]
fn awaited_rhs_and_three_clause_headers_preserve_tdz_and_iteration_closures() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let result,tdz=false,closures=[];async function f(){try{let [x=await x]=[]}catch(error){tdz=error instanceof ReferenceError}for(let [x=await 7]=await [];x<9;x++){closures.push(()=>x)}return closures.map(f=>f()).join(',')}f().then(value=>{result=value});
    "#).unwrap();
    drain(&mut realm);
    check(&mut realm, "result==='7,8'&&tdz&&typeof x==='undefined'");
}

#[test]
fn ordinary_defaults_keep_name_inference_and_awaited_anonymous_values_do_not_gain_names() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let events=[],result;async function f(){let [klass=class{static{events.push(this.name)}},arrow=await(()=>{}),named=function(){},anonymous=await class{static seen=this.name}]=[];return klass.name==='klass'&&arrow.name===''&&named.name==='named'&&anonymous.name===''&&anonymous.seen===''}f().then(value=>{result=value});
    "#).unwrap();
    check(&mut realm, "events.join(',')==='klass'&&result===undefined");
    drain(&mut realm);
    check(&mut realm, "result===true");
}

#[test]
fn rejection_and_normal_close_errors_close_nested_iterators_once_from_inside_out() {
    for reject in [true, false] {
        let mut realm = Realm::default();
        realm
            .eval("let reason={},closing={},events=[],result;")
            .unwrap();
        realm.eval("function source(name,value){return{[Symbol.iterator](){return this},next(){return{done:false,value}},return(){events.push(name);if(name==='inner')throw closing;return{done:true}}}}").unwrap();
        let value = if reject { "undefined" } else { "1" };
        realm.eval(&format!("async function f(){{try{{let [[x=await Promise.reject(reason)]]=source('outer',source('inner',{value}));return false}}catch(error){{return error==={}}}}}f().then(value=>{{result=value}})",if reject {"reason"}else{"closing"})).unwrap();
        drain(&mut realm);
        check(
            &mut realm,
            "result===true&&events.join(',')==='inner,outer'",
        );
    }
}

#[test]
fn step_and_result_errors_skip_current_close_but_close_the_parent_iterator() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let reason={},events=[],results=[],operands=0;
        function inner(kind){return{[Symbol.iterator](){return this},next(){if(kind==='next')throw reason;let value={done:false,value:undefined};Object.defineProperty(value,kind,{get(){throw reason}});return value},return(){events.push('wrong');return{done:true}}}}
        function outer(value){return{[Symbol.iterator](){return this},next(){return{done:false,value}},return(){events.push('outer');return{done:true}}}}
        async function f(kind){try{let [[x=await(operands++,1)]]=outer(inner(kind))}catch(error){return error===reason}}for(const kind of ['next','done','value'])f(kind).then(value=>results.push(value));
    "#).unwrap();
    drain(&mut realm);
    check(
        &mut realm,
        "results.length===3&&results.every(x=>x===true)&&events.join(',')==='outer,outer,outer'&&operands===0",
    );
}

#[test]
fn pending_defaults_keep_cached_next_and_its_environment_live_then_release_them() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),weak,nextWeak,iteratorWeak,result;
        function source(){let keep={};weak=new WeakRef(keep);let next=()=>{keep;return{done:false,value:undefined}};nextWeak=new WeakRef(next);let iterator={next,return(){return{done:true}}};iteratorWeak=new WeakRef(iterator);return{[Symbol.iterator](){return iterator}}}
        async function f(){let [x=await gate]=source();return x}f().then(value=>{result=value});iteratorWeak.deref().next=null;
    "#).unwrap();
    collect(&mut realm);
    check(
        &mut realm,
        "weak.deref()!==undefined&&nextWeak.deref()!==undefined&&iteratorWeak.deref()!==undefined&&result===undefined",
    );
    realm.eval("release(7)").unwrap();
    drain(&mut realm);
    collect(&mut realm);
    check(
        &mut realm,
        "result===7&&weak.deref()===undefined&&nextWeak.deref()===undefined&&iteratorWeak.deref()===undefined",
    );
}

#[test]
fn host_failure_in_a_default_skips_iterator_close_disposal_catch_and_finally() {
    for limited in [false, true] {
        let mut realm = Realm::with_unavailable_host_limits(Limits {
            max_steps: limited.then_some(10_000),
            ..Default::default()
        });
        realm.eval("let state={},closed=0,disposed=0,finalized=0,caught=false,proceed,gate=new Promise(resolve=>{proceed=resolve});const dispose=Symbol.dispose;").unwrap();
        realm.eval("let resource={[dispose](){disposed++}};state.weak=new WeakRef(resource[dispose]);function spin(){while(true){}};let source={[Symbol.iterator](){return this},next(){return{done:false,value:undefined}},return(){closed++;return{done:true}}}").unwrap();
        let tail = if limited {
            "spin()"
        } else {
            "spiteUnavailableHost()"
        };
        realm.eval(&format!("async function f(){{using r=resource;try{{let [x=(await gate,{tail})]=source}}catch{{caught=true}}finally{{finalized++}}}}f().catch(()=>{{caught=true}})")).unwrap();
        realm.eval("delete resource[dispose];proceed()").unwrap();
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
        collect(&mut realm);
        check(
            &mut realm,
            "closed===0&&disposed===0&&finalized===0&&!caught&&state.weak.deref()===undefined",
        );
    }
}

#[test]
fn object_bindings_and_assignment_defaults_resume_operands_and_finalizers() {
    for declaration in [
        "var {x=await(operand++)}={}",
        "let {x=await(operand++)}={}",
        "let x;([x=await(operand++)]=[])",
        "let x;({x=await(operand++)}={})",
    ] {
        let mut realm = Realm::default();
        realm
            .eval("let before=0,operand=0,handled=0,observed=-1")
            .unwrap();
        realm.eval(&format!("async function f(){{before++;try{{{declaration};observed=x}}catch{{handled=1}}finally{{handled=2}}}}f().catch(()=>{{handled=3}})")).unwrap();
        check(
            &mut realm,
            "before===1&&operand===1&&handled===0&&observed===-1",
        );
        drain(&mut realm);
        check(
            &mut realm,
            "before===1&&operand===1&&handled===2&&observed===0",
        );
    }
}
