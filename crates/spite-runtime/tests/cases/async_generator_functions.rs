//! Async-generator function objects and hidden intrinsic metadata (15.6, 27.4–6).

use spite_runtime::{Error, PromiseJobOutcome, Realm, Value};

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}
fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn hidden_constructor_function_and_instance_prototype_graph_has_standard_metadata() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        async function*g(x,y=1){}let FP=Object.getPrototypeOf(g),AGF=FP.constructor,GP=FP.prototype,AP=Object.getPrototypeOf(GP);
        let p=Object.getOwnPropertyDescriptor(g,'prototype'),cp=Object.getOwnPropertyDescriptor(AGF,'prototype'),c=Object.getOwnPropertyDescriptor(GP,'constructor');
        typeof AsyncGeneratorFunction==='undefined'&&typeof AGF==='function'&&typeof FP==='object'&&AGF.name==='AsyncGeneratorFunction'&&AGF.length===1&&g.length===1&&g.name==='g'&&
        Object.getPrototypeOf(AGF)===Function&&AGF.prototype===FP&&Object.getPrototypeOf(FP)===Function.prototype&&GP.constructor===FP&&Object.getPrototypeOf(g.prototype)===GP&&
        !Object.hasOwn(g.prototype,'constructor')&&p.writable&&!p.enumerable&&!p.configurable&&!cp.writable&&!cp.enumerable&&!cp.configurable&&!c.writable&&!c.enumerable&&c.configurable&&
        Object.getPrototypeOf(AP)===Object.prototype&&AP[Symbol.asyncIterator].call(g)===g&&typeof AP[Symbol.asyncDispose]==='function'&&
        Reflect.ownKeys(FP).map(k=>typeof k==='symbol'?k.toString():k).join(',')==='constructor,prototype,Symbol(Symbol.toStringTag)'&&
        Object.prototype.toString.call(g)==='[object AsyncGeneratorFunction]'&&Object.prototype.toString.call(GP)==='[object AsyncGenerator]'&&
        ['next','return','throw'].every(name=>{let d=Object.getOwnPropertyDescriptor(GP,name);return d.writable&&!d.enumerable&&d.configurable&&d.value.name===name&&d.value.length===1})
    "#,
    );
}

#[test]
fn declarations_expressions_and_public_private_methods_are_callable_and_nonconstructible() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        async function*g(){}let o={async*m(x,y=1){}},expression=async function* named(){};
        class C{async*m(){}static async*s(){}async*#p(){}get(){return this.#p}}
        let values=[g,expression,o.m,new C().m,C.s,new C().get()],FP=Object.getPrototypeOf(g),GP=FP.prototype;
        values.every(f=>{let caught=false;try{new f()}catch(e){caught=e instanceof TypeError}let target=false;try{Reflect.construct(Object,[],f)}catch(e){target=e instanceof TypeError}return typeof f==='function'&&caught&&target&&Object.getPrototypeOf(f)===FP&&Object.hasOwn(f,'prototype')&&Object.getPrototypeOf(f.prototype)===GP})&&
        expression.name==='named'&&o.m.name==='m'&&o.m.length===1&&new C().get().name==='#p'&&g instanceof FP.constructor&&Object.create(g.prototype) instanceof g
    "#,
    );
}

#[test]
fn dynamic_constructor_calls_and_construction_share_source_conversion_and_prototype_rules() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let AGF=Object.getPrototypeOf(async function*(){}).constructor,events=[],custom={};
        let N=new Proxy(function(){},{get(t,k,r){if(k==='prototype'){events.push('prototype');return custom}return Reflect.get(t,k,r)}});
        let f=Reflect.construct(AGF,[{toString(){events.push('parameter');return 'x'}},{toString(){events.push('body');return 'yield await x'}}],N);
        let plain=AGF(),built=new AGF('x','yield x'),GP=AGF.prototype.prototype;
        events.join(',')==='parameter,body,prototype'&&Object.getPrototypeOf(f)===custom&&Object.getPrototypeOf(f.prototype)===GP&&f.name==='anonymous'&&f.length===1&&
        plain.toString()==='async function* anonymous(\n) {\n\n}'&&built.toString()==='async function* anonymous(x\n) {\nyield x\n}'&&built instanceof AGF&&built.length===1
    "#,
    );
    check(
        &mut realm,
        r#"
        events=[];let caught=false;try{Reflect.construct(AGF,[{toString(){events.push('parameter');return 'x=await 1'}},{toString(){events.push('body');return ''}}],N)}catch(e){caught=e instanceof SyntaxError}
        caught&&events.join(',')==='parameter,body'&&AGF('yield "\ud800"').toString().includes('\ud800')
    "#,
    );
}

#[test]
fn prototype_methods_reject_unbranded_receivers_with_fresh_intrinsic_promises() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        r#"
        let P=Promise,GP=Object.getPrototypeOf(async function*(){}).prototype,seen=[],promises=[];Promise=null;
        for(let name of ['next','return','throw'])for(let receiver of [undefined,null,7,Symbol(),{},GP,(function*(){})(),new Proxy({}, {})]){
            let promise=GP[name].call(receiver,1);if(!(promise instanceof P))throw 1;promises.push(promise);promise.then(()=>seen.push(false),error=>seen.push(error instanceof TypeError));
        }
        promises.length===24&&new Set(promises).size===24&&seen.length===0
    "#,
    );
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error:?}"),
        }
    }
    check(&mut realm, "seen.length===24&&seen.every(value=>value)");
}

#[test]
fn calls_perform_parameter_setup_before_the_remaining_instance_boundary() {
    let mut realm = Realm::default();
    realm
        .eval("let effects=0,body=0,caught=0,finalized=0;async function*g(x=(effects=3)){body++}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{g()}catch{caught++}finally{finalized++}"),
        Err(Error::Unsupported { .. })
    ));
    check(
        &mut realm,
        "effects===3&&body===0&&caught===0&&finalized===0",
    );
    check(
        &mut realm,
        "async function*h({x}){body++}try{h({get x(){throw 7}})}catch(e){caught=e===7?1:2}finally{finalized++}caught===1&&finalized===1&&body===0",
    );
}

#[test]
fn function_and_method_captures_remain_live_and_unreachable_symbol_cycles_collect() {
    for factory in [
        "(()=>{let held=Symbol();weak=new WeakRef(held);return async function* named(){return held}})()",
        "(()=>{let held=Symbol();weak=new WeakRef(held);return {tag:held,async*g(){return this.tag}}.g})()",
    ] {
        let mut realm = Realm::default();
        realm.eval(&format!("let weak,fn={factory}")).unwrap();
        collect(&mut realm);
        check(&mut realm, "weak.deref()!==undefined");
        realm.eval("fn=null").unwrap();
        collect(&mut realm);
        check(&mut realm, "weak.deref()===undefined");
    }
}
