//! Lazy one-level flattening and nested closing (27.1.3.3.6).

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

#[test]
fn validation_closes_before_next_lookup_and_acquisition_errors_do_not_close() {
    for callback in [
        "undefined",
        "null",
        "true",
        "7",
        "7n",
        "'x'",
        "Symbol()",
        "{}",
    ] {
        for close in ["return {};", "return 7;", "throw 8;"] {
            check(&format!(
                "let closed=0,i={{get next(){{throw 9;}},return(){{closed++;{close}}}}},caught=false;try{{Iterator.prototype.flatMap.call(i,{callback});}}catch(e){{caught=e instanceof TypeError;}}caught && closed===1"
            ));
        }
    }
    for receiver in ["undefined", "null", "true", "1", "1n", "'x'", "Symbol()"] {
        assert!(matches!(
            Realm::default().eval(&format!(
                "Iterator.prototype.flatMap.call({receiver},()=>[])"
            )),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    check(
        "let closed=0,i={get next(){throw 7;},return(){closed++;return {};}};let caught=false;try{Iterator.prototype.flatMap.call(i,()=>[]);}catch(e){caught=e===7;}caught && closed===0",
    );
}

#[test]
fn outer_capture_is_eager_and_inner_acquisition_and_steps_are_lazy_and_cached() {
    check(
        "let log='',n=0,seen=[],context,argc,outer,inner,innerGets=0,innerCalls=0,i={get next(){log+='o';return function(){'use strict';outer=this;argc=arguments.length;return {done:n===3,value:n++};};}};let h=Iterator.prototype.flatMap.call(i,function(value,index){'use strict';context=this;seen.push(value,index,arguments.length);let k=0,j={get next(){innerGets++;return function(){'use strict';inner=this;innerCalls++;return {done:k===2,value:value*10+k++};};}};return {[Symbol.iterator](){'use strict';log+=arguments.length===0?'m':'X';return j;}};},7);let eager=log==='o' && n===0;let a=h.next(7);let first=eager && a.value===0 && !a.done && innerGets===1 && innerCalls===1 && n===1;let b=h.next(),c=h.next(),d=h.next(),e=h.next(),f=h.next(),end=h.next();first && b.value===1 && c.value===10 && d.value===11 && e.value===20 && f.value===21 && end.done && seen.join(',')==='0,0,2,1,1,2,2,2,2' && context===undefined && argc===0 && outer===i && innerGets===3 && innerCalls===9 && log==='ommm'",
    );
    check(
        "let i=[1,2,3].values(),h=i.flatMap(x=>[x,x]);i.next();h.toArray().join(',')==='2,2,3,3'",
    );
    check(
        "let context,h=[7].values().flatMap(function(){context=this;return [];});h.next().done && context===globalThis",
    );
    check(
        "let gets=0,calls=0,j={};Object.defineProperty(j,'next',{configurable:true,get(){gets++;return ()=>({done:calls++>0,value:7});}});let h=[0].values().flatMap(()=>j);h.next();Object.defineProperty(j,'next',{value:()=>{throw 8;}});h.next().done && gets===1 && calls===2",
    );
}

#[test]
fn flattening_accepts_direct_or_iterable_objects_rejects_all_primitives_and_is_one_level() {
    for hook in ["undefined", "null"] {
        check(&format!(
            "let n=0,j={{[Symbol.iterator]:{hook},next:()=>({{done:n++>0,value:7}})}};[0].values().flatMap(()=>j).toArray().join(',')==='7'"
        ));
    }
    check(
        "let a=[1],original={},h=[7].values().flatMap(()=>[a,original,undefined]);let first=h.next(),second=h.next(),third=h.next();first.value===a && second.value===original && !third.done && third.value===undefined && h.next().done",
    );
    check("[7].values().flatMap(()=>Object('ab')).toArray().join(',')==='a,b'");
    for value in ["undefined", "null", "true", "0", "0n", "'ab'", "Symbol()"] {
        check(&format!(
            "let read=false,closed=0;Object.defineProperty(String.prototype,Symbol.iterator,{{get(){{read=true;throw 9;}}}});let h=Iterator.prototype.flatMap.call({{next:()=>({{done:false,value:7}}),return(){{closed++;return {{}};}}}},()=>{value});let caught=false;try{{h.next();}}catch(e){{caught=e instanceof TypeError;}}caught && !read && closed===1 && h.next().done"
        ));
    }
    check(
        "let count=0,h=[0,1,2,3].values().flatMap((value,index)=>{count+=index;return index%2?[value,index]:[];});h.toArray().join(',')==='1,1,3,3' && count===6",
    );
}

#[test]
fn mapper_and_inner_acquisition_throws_close_only_outer_preserving_incoming_error() {
    for mapper in [
        "()=>{throw sentinel;}",
        "()=>({get [Symbol.iterator](){throw sentinel;}})",
        "()=>({[Symbol.iterator](){throw sentinel;}})",
        "()=>({get next(){throw sentinel;}})",
    ] {
        for close in ["return {};", "return 7;", "throw 8;"] {
            check(&format!(
                "let sentinel={{}},closed=0,h=Iterator.prototype.flatMap.call({{next:()=>({{done:false,value:7}}),return(){{closed++;{close}}}}},{mapper}),caught=false;try{{h.next();}}catch(e){{caught=e===sentinel;}}caught && closed===1 && h.next().done && h.return().done && closed===1"
            ));
        }
    }
    for inner in ["{[Symbol.iterator]:7}", "{[Symbol.iterator]:()=>7}"] {
        check(&format!(
            "let closed=0,h=Iterator.prototype.flatMap.call({{next:()=>({{done:false,value:7}}),return(){{closed++;return {{}};}}}},()=>({inner}));let caught=false;try{{h.next();}}catch(e){{caught=e instanceof TypeError;}}caught && closed===1 && h.next().done"
        ));
    }
}

#[test]
fn inner_step_throws_close_outer_only_while_outer_step_throws_skip_all_cleanup() {
    for next in [
        "()=>{throw 7;}",
        "()=>7",
        "()=>({get done(){throw 7;}})",
        "()=>({done:false,get value(){throw 7;}})",
        "7",
    ] {
        check(&format!(
            "let outer=0,inner=0,j={{next:{next},return(){{inner++;throw 8;}}}},h=Iterator.prototype.flatMap.call({{next:()=>({{done:false,value:7}}),return(){{outer++;throw 9;}}}},()=>j),caught=false;try{{h.next();}}catch(e){{caught=e===7 || e instanceof TypeError;}}caught && outer===1 && inner===0 && h.next().done && h.return().done"
        ));
        check(&format!(
            "let closed=0,calls=0,h=Iterator.prototype.flatMap.call({{next:{next},return(){{closed++;throw 8;}}}},()=>{{calls++;return [];}}),caught=false;try{{h.next();}}catch(e){{caught=e===7 || e instanceof TypeError;}}caught && closed===0 && calls===0 && h.next().done"
        ));
    }
    check(
        "let h=[7].values().flatMap(()=>({next:()=>({done:true,get value(){throw 7;}}),get return(){throw 8;}}));h.next().done && h.return().done",
    );
}

#[test]
fn return_before_start_completes_before_outer_cleanup_and_never_calls_mapper() {
    check(
        "let closed=0,calls=0,steps=0,reentered=false,h,i={next(){steps++;throw 7;},return(){closed++;reentered=h.next().done && h.return().done;return {};}};h=Iterator.prototype.flatMap.call(i,()=>{calls++;throw 8;});let r=h.return(9);r.done && r.value===undefined && calls===0 && steps===0 && closed===1 && reentered && h.next().done",
    );
}

#[test]
fn yielded_return_closes_live_inner_then_outer_with_error_precedence_and_reentry() {
    check(
        "let log='',reentries=0,h,inner={next:()=>({done:false,value:7}),return(){throw 8;}},outer={next:()=>({done:false,value:7}),return(){throw 9;}};h=Iterator.prototype.flatMap.call(outer,()=>inner);h.next();for(let pair of [[inner,'i'],[outer,'o']])Object.defineProperty(pair[0],'return',{get(){log+=pair[1];return function(){'use strict';if(this!==pair[0] || arguments.length!==0)throw 10;try{h.next();}catch(e){if(e instanceof TypeError)reentries++;}try{h.return();}catch(e){if(e instanceof TypeError)reentries++;}return {};};}});let r=h.return();r.done && r.value===undefined && log==='io' && reentries===4 && h.next().done && h.return().done && log==='io'",
    );
    for inner in ["return 7;", "throw sentinel;", "return {};"] {
        for outer in ["return 7;", "throw other;", "return {};"] {
            check(&format!(
                "let log='',sentinel={{}},other={{}},j={{next:()=>({{done:false,value:7}}),return(){{log+='i';{inner}}}}},h=Iterator.prototype.flatMap.call({{next:()=>({{done:false,value:7}}),return(){{log+='o';{outer}}}}},()=>j);h.next();let result,err;try{{result=h.return();}}catch(e){{err=e;}}let expected='{inner}'==='throw sentinel;'?err===sentinel:('{inner}'==='return 7;'?err instanceof TypeError:('{outer}'==='throw other;'?err===other:('{outer}'==='return 7;'?err instanceof TypeError:result.done)));expected && log==='io' && h.next().done && h.return().done && log==='io'"
            ));
        }
    }
    check(
        "let sentinel={},log='',j={next:()=>({done:false,value:7}),get return(){log+='i';throw sentinel;}},h=Iterator.prototype.flatMap.call({next:()=>({done:false,value:7}),return(){log+='o';throw 8;}},()=>j);h.next();let caught=false;try{h.return();}catch(e){caught=e===sentinel;}caught && log==='io' && h.next().done",
    );
}

#[test]
fn reentry_during_mapper_or_inner_step_is_rejected_and_follows_closing_rules() {
    for mapper in [
        "()=>h.next()",
        "()=>({[Symbol.iterator]:()=>h.return()})",
        "()=>({next:()=>h.next(),return(){inner++;return {};}})",
    ] {
        check(&format!(
            "let outer=0,inner=0,h;h=Iterator.prototype.flatMap.call({{next:()=>({{done:false,value:7}}),return(){{outer++;return {{}};}}}},{mapper});let caught=false;try{{h.next();}}catch(e){{caught=e instanceof TypeError;}}caught && outer===1 && inner===0 && h.next().done"
        ));
    }
    check(
        "let h,closed=0;h=Iterator.prototype.flatMap.call({next:()=>h.next(),return(){closed++;return {}; }},()=>[]);let caught=false;try{h.next();}catch(e){caught=e instanceof TypeError;}caught && closed===0 && h.next().done",
    );
    check(
        "let h,caught=0;h=[7].values().flatMap(()=>{try{h.next();}catch(e){if(e instanceof TypeError)caught++;}return {next(){try{h.return();}catch(e){if(e instanceof TypeError)caught++;}return {done:false,value:8};}};});h.next().value===8 && caught===2",
    );
}

#[test]
fn outer_mapper_cached_next_and_active_inner_captures_survive_collection_and_release() {
    let mut realm = Realm::default();
    let handles = ["i", "i.next", "mapper", "j", "j.next"].map(|expression| {
        if expression == "i" {
            realm.eval("let i={next:()=>({done:false,value:7})},j={next:()=>({done:false,value:8})},mapper=()=>j,h=Iterator.prototype.flatMap.call(i,mapper);").unwrap();
        }
        let Value::Object(handle) = realm.eval(expression).unwrap() else {panic!("capture")};handle
    });
    assert_eq!(realm.eval("h.next().value"), Ok(Value::Number(8.0)));
    realm
        .eval("delete i.next;delete j.next;i=null;j=null;mapper=null;")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(handles.iter().all(|h| realm.inspect_object(h).is_ok()));
    assert_eq!(realm.eval("h.next().value"), Ok(Value::Number(8.0)));
    realm.eval("h.return()").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 1);
    assert!(handles.iter().all(|h| realm.inspect_object(h).is_err()));
    realm
        .eval("h=null;let f=Iterator.prototype.flatMap;delete Iterator.prototype.flatMap;")
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("f.call([7].values(),x=>[x]).next().value"),
        Ok(Value::Number(7.0))
    );
}

#[test]
fn metadata_and_shared_helper_brand_are_standard() {
    check(
        "let f=Iterator.prototype.flatMap,d=Object.getOwnPropertyDescriptor(Iterator.prototype,'flatMap'),h=[7].values().flatMap(x=>[x]),p=Object.getPrototypeOf(h),c=Iterator.concat();f.name==='flatMap' && f.length===1 && Object.getPrototypeOf(f)===Function.prototype && !Object.hasOwn(f,'prototype') && d.value===f && d.writable && !d.enumerable && d.configurable && h instanceof Iterator && Object.getPrototypeOf(p)===Iterator.prototype && Object.prototype.toString.call(h)==='[object Iterator Helper]' && h.next===c.next && h.return===c.return && h[Symbol.iterator]()===h",
    );
    assert!(matches!(
        Realm::default().eval("new Iterator.prototype.flatMap(()=>[])"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    check(
        "let h=[7].values().flatMap(x=>[x]),wrong=Object.create(h),caught=false;try{h.next.call(wrong);}catch(e){caught=e instanceof TypeError;}Object.setPrototypeOf(h,null);caught && Iterator.concat().next.call(h).value===7",
    );
}

#[test]
fn large_default_inputs_skip_empty_inners_iteratively_and_host_failures_skip_cleanup() {
    check(
        "let n=0,h=Iterator.prototype.flatMap.call({next:()=>({done:n===10000,value:n++})},(value,index)=>index===9999?[value,index]:[]);h.toArray().join(',')==='9999,9999' && n===10001",
    );
    check(
        "let n=0,h=[7].values().flatMap(()=>({next:()=>({done:n===10000,value:n++})}));let a=h.toArray();a.length===10000 && a[9999]===9999",
    );
    for mapper in [
        "()=>[]",
        "()=>({next:()=>{while(true){};},return(){flag=4;return {};}})",
    ] {
        let mut realm = Realm::new(Limits {
            max_steps: Some(5000),
            ..Limits::default()
        });
        realm.eval(&format!("let flag=0,h=Iterator.prototype.flatMap.call({{next:()=>({{done:false,value:7}}),return(){{flag=3;return {{}};}}}},{mapper});")).unwrap();
        assert!(matches!(
            realm.eval("try{h.next();}catch{flag=1;}finally{flag=2;}"),
            Err(Error::Limit { .. })
        ));
        assert_eq!(
            realm.eval("flag===0 && h.next().done && h.return().done"),
            Ok(Value::Boolean(true))
        );
    }
    for mapper in [
        "()=>Function('function* gap(){}')",
        "()=>({[Symbol.iterator]:()=>Function('function* gap(){}')})",
        "()=>({next:()=>Function('function* gap(){}'),return(){flag=4;return {};}})",
    ] {
        let mut realm = Realm::default();
        realm.eval(&format!("let flag=0,h=Iterator.prototype.flatMap.call({{next:()=>({{done:false,value:7}}),return(){{flag=3;return {{}};}}}},{mapper});")).unwrap();
        assert!(matches!(
            realm.eval("try{h.next();}catch{flag=1;}finally{flag=2;}"),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(
            realm.eval("flag===0 && h.next().done && h.return().done"),
            Ok(Value::Boolean(true))
        );
    }
    let mut realm = Realm::default();
    realm.eval("let flag=0,h=Iterator.prototype.flatMap.call({next:()=>({done:false,value:7}),return(){flag=3;return {}; }},()=>({next:()=>({done:false,value:8}),return:()=>Function('function* gap(){}')}));h.next();").unwrap();
    assert!(matches!(
        realm.eval("try{h.return();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && h.next().done && h.return().done"),
        Ok(Value::Boolean(true))
    );
}
