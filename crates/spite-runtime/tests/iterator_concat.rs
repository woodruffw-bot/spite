//! Lazy concat capture, resume, closing, branding, and GC (27.1.3.2.1).

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn type_error(source: &str) {
    assert!(
        matches!(
            Realm::default().eval(source),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ),
        "{source}"
    );
}

#[test]
fn ordered_method_capture_is_eager_but_iterator_opening_and_next_capture_are_lazy() {
    check(
        "let log='',reads=0,calls=0,receiver,argc,i={};Object.defineProperty(i,'next',{configurable:true,get:()=>{reads++;log+='n';return function(){'use strict';receiver=this;argc=arguments.length;return {done:calls++>0,value:7};};}});let a={};Object.defineProperty(a,Symbol.iterator,{configurable:true,get:()=>{log+='g';return function(){'use strict';log+=this===a?'c':'X';return i;};}});let b={};Object.defineProperty(b,Symbol.iterator,{get:()=>{log+='h';return ()=>{log+='d';return [].values();};}});let w=Iterator.concat.call(null,a,b);let eager=log==='gh' && reads===0;Object.defineProperty(a,Symbol.iterator,{value:()=>{throw 9;}});let first=w.next(99);Object.defineProperty(i,'next',{value:()=>{throw 10;}});eager && first.value===7 && !first.done && w.next().done && log==='ghcnd' && reads===1 && receiver===i && argc===0",
    );
    check(
        "let reads=0,a={};Object.defineProperty(a,Symbol.iterator,{get:()=>{reads++;return ()=>{throw 7;};}});let caught=false;try{Iterator.concat(a,null,{get [Symbol.iterator](){throw 8;}});}catch(e){caught=e instanceof TypeError;}caught && reads===1",
    );
}

#[test]
fn primitives_and_missing_methods_are_rejected_without_opening_sources() {
    for value in ["undefined", "null", "true", "1", "1n", "'ab'", "Symbol()"] {
        type_error(&format!(
            "Object.defineProperty(Number.prototype,Symbol.iterator,{{get:()=>{{throw 7;}}}});Object.defineProperty(String.prototype,Symbol.iterator,{{get:()=>{{throw 8;}}}});Iterator.concat({value})"
        ));
    }
    for method in ["undefined", "null", "true", "7", "7n", "'x'", "Symbol()"] {
        type_error(&format!(
            "Iterator.concat({{[Symbol.iterator]:{method},next:()=>({{done:true}})}})"
        ));
    }
    check(
        "let w=Iterator.concat(new String('ab'),[],[3]);Array.from(w).join(',')==='a,b,3' && w.next().done",
    );
}

#[test]
fn results_are_fresh_and_done_skips_value_while_empty_sources_advance_iteratively() {
    check(
        "let done={done:true,get value(){throw 7;}},value={},result={done:false,value},n=0,a={[Symbol.iterator]:()=>({next:()=>n++===0?result:done,return:()=>{throw 8;}})},w=Iterator.concat([],a,[],[9]);let first=w.next(),second=w.next(),last=w.next();first!==result && first.value===value && !first.done && second.value===9 && !second.done && last.value===undefined && last.done && w.next()!==last && w.next().done && w.return().done",
    );
    check(
        "let w=Iterator.concat(),a=w.next(),b=w.next(),c=w.return();a!==b && b!==c && a.done && a.value===undefined && Reflect.ownKeys(a).join(',')==='value,done' && Object.getPrototypeOf(a)===Object.prototype",
    );
    check(
        "let sources=[];for(let i=0;i<2000;i++)sources.push([]);sources.push([7]);let w=Iterator.concat(...sources);w.next().value===7 && w.next().done",
    );
}

#[test]
fn step_and_opening_errors_complete_without_closing_and_never_resume_again() {
    for next in [
        "()=>{throw 7;}",
        "()=>7",
        "()=>({get done(){throw 7;}})",
        "()=>({done:false,get value(){throw 7;}})",
    ] {
        check(&format!(
            "let calls=0,closed=0,opens=0,a={{[Symbol.iterator](){{opens++;return {{next(){{calls++;return ({next})();}},return(){{closed++;throw 8;}}}};}}}},w=Iterator.concat(a),caught=false;try{{w.next();}}catch(e){{caught=true;}}caught && calls===1 && closed===0 && opens===1 && w.next().done && w.return().done && calls===1"
        ));
    }
    for method in [
        "()=>7",
        "()=>{throw 7;}",
        "()=>({get next(){throw 7;},return(){throw 8;}})",
    ] {
        check(&format!(
            "let calls=0,a={{[Symbol.iterator](){{calls++;return ({method})();}}}},w=Iterator.concat(a),caught=false;try{{w.next();}}catch(e){{caught=true;}}caught && calls===1 && w.next().done && w.return().done && calls===1"
        ));
    }
}

#[test]
fn return_closes_only_the_yielded_source_once_with_zero_arguments() {
    check(
        "let opens=0,closed=0,receiver,argc,i={next:()=>({value:7,done:false}),return:function(){'use strict';closed++;receiver=this;argc=arguments.length;return {get done(){throw 8;},get value(){throw 9;}};}},a={[Symbol.iterator](){opens++;return i;}},b={[Symbol.iterator](){throw 10;}},w=Iterator.concat(a,b),first=w.next(),last=w.return(99);first.value===7 && !first.done && last.done && last.value===undefined && receiver===i && argc===0 && opens===1 && closed===1 && w.next().done && w.return().done && closed===1",
    );
    check(
        "let calls=0,a={[Symbol.iterator](){calls++;throw 7;}},w=Iterator.concat(a);w.return(99).done && w.next().done && w.return().done && calls===0",
    );
    check(
        "let closed=0,w=Iterator.concat({[Symbol.iterator]:()=>({next:()=>({done:true}),return:()=>{closed++;throw 7;}})});w.next().done && w.return().done && closed===0",
    );
    check(
        "let closed='',a={[Symbol.iterator]:()=>({next:()=>({done:true}),return:()=>{closed+='a';throw 7;}})},b={[Symbol.iterator]:()=>({next:()=>({done:false,value:8}),return:()=>{closed+='b';return {};}})},w=Iterator.concat(a,b);w.next().value===8 && w.return().done && closed==='b'",
    );
}

#[test]
fn return_errors_complete_the_helper_and_do_not_reopen_or_reclose_sources() {
    for method in ["7", "()=>7", "()=>{throw 7;}"] {
        check(&format!(
            "let closed=0,i={{next:()=>({{done:false,value:8}}),get return(){{closed++;return {method};}}}},w=Iterator.concat({{[Symbol.iterator]:()=>i}});w.next();let caught=false;try{{w.return();}}catch(e){{caught=true;}}caught && closed===1 && w.return().done && w.next().done && closed===1"
        ));
    }
    check(
        "let i={next:()=>({done:false,value:7})},w=Iterator.concat({[Symbol.iterator]:()=>i});w.next();w.return().done && w.next().done",
    );
}

#[test]
fn executing_next_and_return_reject_reentry_but_caught_reentry_can_continue() {
    for action in ["w.next()", "w.return()"] {
        check(&format!(
            "let calls=0,w=Iterator.concat({{[Symbol.iterator]:()=>({{next(){{calls++;{action};return {{done:false}};}}}})}}),caught=false;try{{w.next();}}catch(e){{caught=e instanceof TypeError;}}caught && calls===1 && w.next().done"
        ));
        check(&format!(
            "let closed=0,w=Iterator.concat({{[Symbol.iterator]:()=>({{next:()=>({{done:false}}),return(){{closed++;{action};return {{}};}}}})}});w.next();let caught=false;try{{w.return();}}catch(e){{caught=e instanceof TypeError;}}caught && closed===1 && w.return().done && w.next().done"
        ));
    }
    check(
        "let caught=false,w=Iterator.concat({[Symbol.iterator]:()=>({next(){try{w.next();}catch(e){caught=e instanceof TypeError;}return {done:false,value:7};}})});w.next().value===7 && caught && w.next().value===7",
    );
}

#[test]
fn helper_brands_prototypes_and_complete_constructor_reflection_are_standard() {
    for method in ["next", "return"] {
        for receiver in [
            "undefined",
            "null",
            "1",
            "{}",
            "Iterator.prototype",
            "Object.getPrototypeOf(w)",
            "Object.create(w)",
            "Iterator.from({})",
            "[].values()",
        ] {
            type_error(&format!(
                "let w=Iterator.concat();w.{method}.call({receiver})"
            ));
        }
        type_error(&format!("let w=Iterator.concat();new w.{method}()"));
        check(&format!(
            "let p=Object.getPrototypeOf(Iterator.concat()),d=Object.getOwnPropertyDescriptor(p,'{method}');d.value.name==='{method}' && d.value.length===0 && d.writable && !d.enumerable && d.configurable && !Object.hasOwn(d.value,'prototype') && Object.getPrototypeOf(d.value)===Function.prototype"
        ));
    }
    check(
        "let w=Iterator.concat(),p=Object.getPrototypeOf(w),d=Object.getOwnPropertyDescriptor(p,Symbol.toStringTag),keys=Reflect.ownKeys(p);Object.getPrototypeOf(p)===Iterator.prototype && keys.length===3 && keys[0]==='next' && keys[1]==='return' && keys[2]===Symbol.toStringTag && d.value==='Iterator Helper' && !d.writable && !d.enumerable && d.configurable && w instanceof Iterator && Iterator.from(w)===w && Object.prototype.toString.call(w)==='[object Iterator Helper]' && (Object.freeze(p),Object.isFrozen(p))",
    );
    check(
        "let w=Iterator.concat([7]),next=w.next;Object.setPrototypeOf(w,null);Object.freeze(w);next.call(w).value===7 && next.call(w).done",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(Iterator,'concat');Iterator.concat.name==='concat' && Iterator.concat.length===0 && d.value===Iterator.concat && d.writable && !d.enumerable && d.configurable && Reflect.ownKeys(Iterator).join(',')==='length,name,concat,from,prototype' && Object.keys(Iterator).length===0 && (Object.freeze(Iterator),Object.isFrozen(Iterator))",
    );
    type_error("new Iterator.concat()");
}

#[test]
fn captures_and_active_next_survive_collection_and_are_released_on_completion() {
    let mut realm = Realm::default();
    let Value::Object(source) = realm.eval("let inner={next:()=>({value:7,done:false})},a={[Symbol.iterator]:()=>inner},w=Iterator.concat(a);a").unwrap() else { panic!("source") };
    let Value::Object(method) = realm.eval("a[Symbol.iterator]").unwrap() else {
        panic!("method")
    };
    let Value::Object(next) = realm.eval("inner.next").unwrap() else {
        panic!("next")
    };
    realm
        .eval("delete a[Symbol.iterator];a=null;delete Iterator.concat;")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(&source).is_ok() && realm.inspect_object(&method).is_ok());
    assert_eq!(realm.eval("w.next().value"), Ok(Value::Number(7.0)));
    realm.eval("delete inner.next;inner=null").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(&next).is_ok());
    assert_eq!(realm.eval("w.next().value"), Ok(Value::Number(7.0)));
    realm.eval("w.return()").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 1);
    assert!(
        realm.inspect_object(&source).is_err()
            && realm.inspect_object(&method).is_err()
            && realm.inspect_object(&next).is_err()
    );
    realm.eval("w=null").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
}

#[test]
fn host_failures_complete_resumes_skip_handlers_and_do_not_run_cleanup() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0,w=Iterator.concat({[Symbol.iterator]:()=>({next(){while(true){}},return(){flag=3;return {};}})})").unwrap();
    assert!(matches!(
        realm.eval("try{w.next();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && w.next().done && w.return().done"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::default();
    realm.eval("let flag=0,w=Iterator.concat({[Symbol.iterator]:()=>({next:()=>Function('class C extends Object{}'),return:()=>{flag=3;return {};}})})").unwrap();
    assert!(matches!(
        realm.eval("try{w.next();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && w.next().done"),
        Ok(Value::Boolean(true))
    );
}
