//! Iterator.from acquisition and exact native wrapper forwarding (27.1.3.2.2).

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

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
fn iterable_and_direct_inputs_preserve_intrinsic_instances_and_wrap_other_iterators() {
    check(
        "let i=[1,2].values();Iterator.from(i)===i && Array.from(Iterator.from([1,2])).join(',')==='1,2' && Array.from(Iterator.from('a\u{1f600}')).length===2 && Array.from(Iterator.from(new String('ab'))).join(',')==='a,b'",
    );
    for method in ["undefined", "null"] {
        check(&format!(
            "let n=0,i={{[Symbol.iterator]:{method},next(){{return {{value:n++,done:n>3}};}}}},w=Iterator.from(i);w!==i && w instanceof Iterator && w[Symbol.iterator]()===w && Array.from(w).join(',')==='0,1,2' && Object.getPrototypeOf(Object.getPrototypeOf(w))===Iterator.prototype"
        ));
    }
    check(
        "let reads=0,i=Object.create(Iterator.prototype);Object.defineProperty(i,'next',{get:()=>{reads++;return 7;}});Iterator.from(i)===i && reads===1",
    );
    check(
        "let I=Iterator,from=I.from,i=[].values();Object.defineProperty(I,Symbol.hasInstance,{get:()=>{throw 7;}});globalThis.Iterator=null;from.call({toString:()=>{throw 8;}},i)===i",
    );
}

#[test]
fn primitive_rejection_precedes_hooks_and_string_hooks_keep_original_receivers() {
    for primitive in ["undefined", "null", "true", "1", "1n", "Symbol()"] {
        type_error(&format!(
            "Object.defineProperty(Number.prototype,Symbol.iterator,{{get:()=>{{throw 7;}}}});Object.defineProperty(Boolean.prototype,Symbol.iterator,{{get:()=>{{throw 8;}}}});Object.defineProperty(BigInt.prototype,Symbol.iterator,{{get:()=>{{throw 9;}}}});Object.defineProperty(Symbol.prototype,Symbol.iterator,{{get:()=>{{throw 10;}}}});Iterator.from({primitive})"
        ));
    }
    check(
        "let original=String.prototype[Symbol.iterator],receiver;Object.defineProperty(String.prototype,Symbol.iterator,{get:function(){'use strict';receiver=this;return original;}});Iterator.from('abc');receiver==='abc'",
    );
    check(
        "let receiver,i={next:()=>({done:true})};String.prototype[Symbol.iterator]=function(){'use strict';receiver=this;return i;};let w=Iterator.from('abc');receiver==='abc' && w!==i && w instanceof Iterator",
    );
    type_error("String.prototype[Symbol.iterator]=null;Iterator.from('abc')");
    type_error("Iterator.from({[Symbol.iterator]:0,next:()=>({done:true})})");
    type_error("Iterator.from({[Symbol.iterator]:()=>7})");
}

#[test]
fn acquisition_order_and_cached_next_do_not_read_return_or_coerce_arguments() {
    check(
        "let log='',receiver,count,reads=0,i={};Object.defineProperty(i,'next',{configurable:true,get:()=>{reads++;log+='n';return function(){'use strict';receiver=this;count=arguments.length;return 7;};}});Object.defineProperty(i,'return',{get:()=>{throw 8;}});let source={};Object.defineProperty(source,Symbol.iterator,{get:()=>{log+='g';return function(){'use strict';log+='c';return this===source?i:null;};}});let w=Iterator.from(source,{valueOf:()=>{throw 9;}});Object.defineProperty(i,'next',{value:()=>{throw 10;}});w.next({valueOf:()=>{throw 11;}})===7 && w.next()===7 && reads===1 && receiver===i && count===0 && log==='gcn'",
    );
    assert_eq!(
        Realm::default()
            .eval("let i={};Object.defineProperty(i,'next',{get:()=>{throw 7;}});Iterator.from(i)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    check("let w=Iterator.from({});typeof w.next==='function' && w.return().done");
    type_error("Iterator.from({next:7}).next()");
}

#[test]
fn next_forwards_results_without_validation_or_completion_tracking() {
    check(
        "let result={};Object.defineProperty(result,'done',{get:()=>{throw 7;}});Object.defineProperty(result,'value',{get:()=>{throw 8;}});let w=Iterator.from({next:()=>result});w.next()===result",
    );
    for result in ["undefined", "null", "7", "'x'"] {
        check(&format!(
            "Iterator.from({{next:()=>{result}}}).next()==={result}"
        ));
    }
    check(
        "let calls=0,w=Iterator.from({next:()=>({done:calls++===0,value:calls})});w.next().done && !w.next().done && calls===2",
    );
    check(
        "let calls=0,w,i={next(){calls++;return calls===1?w.next():{value:calls,done:false};}};w=Iterator.from(i);w.next().value===2 && calls===2",
    );
    check(
        "let calls=0,w=Iterator.from({next(){calls++;if(calls===1)throw 7;return {value:8,done:false};}}),caught=false;try{w.next();}catch(e){caught=e===7;}caught && w.next().value===8 && calls===2",
    );
}

#[test]
fn return_is_looked_up_on_each_call_and_forwards_without_argument_or_result_checks() {
    check(
        "let reads=0,receiver,count,i={next:()=>7};Object.defineProperty(i,'return',{get:()=>{reads++;return function(){'use strict';receiver=this;count=arguments.length;return reads;};}});let w=Iterator.from(i);reads===0 && w.return(99)===1 && w.return()===2 && reads===2 && receiver===i && count===0 && w.next()===7",
    );
    for absent in ["undefined", "null"] {
        check(&format!(
            "let w=Iterator.from({{return:{absent}}}),a=w.return(99),b=w.return();a!==b && a.value===undefined && a.done && Object.getPrototypeOf(a)===Object.prototype && Reflect.ownKeys(a).join(',')==='value,done'"
        ));
    }
    type_error("Iterator.from({return:7}).return()");
    assert_eq!(Realm::default().eval("let i={};Object.defineProperty(i,'return',{get:()=>{throw 7;}});Iterator.from(i).return()"), Err(Error::Thrown(Value::Number(7.0))));
    check("let i={},w=Iterator.from(i);w.return().done && (i.return=()=>99,w.return()===99)");
    check(
        "let calls=0,w=Iterator.from({next:()=>7,return(){calls++;if(calls===1)throw 8;return 9;}}),caught=false;try{w.return();}catch(e){caught=e===8;}caught && w.return()===9 && w.next()===7 && calls===2",
    );
}

#[test]
fn wrappers_have_own_brands_independent_of_prototypes_and_metadata_is_standard() {
    for method in ["next", "return"] {
        for receiver in [
            "undefined",
            "null",
            "true",
            "1",
            "1n",
            "'x'",
            "Symbol()",
            "{}",
            "Object.create(w)",
            "Iterator.prototype",
            "Object.getPrototypeOf(w)",
            "[].values()",
        ] {
            type_error(&format!(
                "let w=Iterator.from({{next:()=>7}});w.{method}.call({receiver})"
            ));
        }
        check(&format!(
            "let w=Iterator.from({{next:()=>7}}),p=Object.getPrototypeOf(w),d=Object.getOwnPropertyDescriptor(p,'{method}');d.value.name==='{method}' && d.value.length===0 && d.writable && !d.enumerable && d.configurable && !Object.hasOwn(d.value,'prototype') && Object.getPrototypeOf(d.value)===Function.prototype"
        ));
        type_error(&format!(
            "let w=Iterator.from({{next:()=>7}});new w.{method}()"
        ));
    }
    check(
        "let w=Iterator.from({next:()=>7}),next=w.next;Object.setPrototypeOf(w,null);Object.freeze(w);next.call(w)===7 && Reflect.ownKeys(w).length===0",
    );
    check(
        "let w=Iterator.from({}),p=Object.getPrototypeOf(w);Reflect.ownKeys(p).join(',')==='next,return' && Object.prototype.toString.call(w)==='[object Iterator]' && (Object.freeze(p),Object.isFrozen(p))",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(Iterator,'from');Iterator.from.name==='from' && Iterator.from.length===1 && d.value===Iterator.from && d.writable && !d.enumerable && d.configurable && !Object.hasOwn(Iterator.from,'prototype')",
    );
    type_error("new Iterator.from()");
}

#[test]
fn cached_callables_and_sources_survive_collection_after_public_deletion() {
    let mut realm = Realm::default();
    let Value::Object(source) = realm
        .eval("let source={next:()=>7},w=Iterator.from(source);source")
        .unwrap()
    else {
        panic!("source")
    };
    let Value::Object(next) = realm.eval("source.next").unwrap() else {
        panic!("next")
    };
    realm
        .eval("delete source.next;source=null;delete Iterator.from;")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(realm.inspect_object(&source).is_ok());
    assert!(realm.inspect_object(&next).is_ok());
    assert_eq!(realm.eval("w.next()"), Ok(Value::Number(7.0)));
    realm.eval("w=null").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert!(realm.inspect_object(&source).is_err());
    assert!(realm.inspect_object(&next).is_err());
}

#[test]
fn host_failures_skip_handlers_and_recursive_next_restores_native_call_state() {
    let mut realm = Realm::default();
    realm
        .eval("let flag=0,w=Iterator.from({next:()=>Function('class C extends Object{}'),return:()=>{flag=3;}})")
        .unwrap();
    assert!(matches!(
        realm.eval("try{w.next();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut realm = Realm::default();
            realm
                .eval("let recurse=true,w=Iterator.from({next:()=>recurse?w.next():7}),flag=0")
                .unwrap();
            assert!(matches!(
                realm.eval("try{w.next();}catch{flag=1;}finally{flag=2;}"),
                Err(Error::Limit { .. })
            ));
            assert_eq!(
                realm.eval("flag===0 && (recurse=false,w.next()===7)"),
                Ok(Value::Boolean(true))
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
