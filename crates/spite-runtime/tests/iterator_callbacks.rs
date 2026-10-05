//! Lazy direct map/filter and Iterator Helper closing (27.1.2, 27.1.3.3.4/8).

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
fn validation_closes_before_next_lookup_and_rejects_primitive_receivers() {
    for method in ["map", "filter"] {
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
                    "let closed=0,i={{get next(){{throw 9;}},return(){{closed++;{close}}}}},caught=false;try{{Iterator.prototype.{method}.call(i,{callback});}}catch(e){{caught=e instanceof TypeError;}}caught && closed===1"
                ));
            }
        }
        for receiver in ["undefined", "null", "true", "1", "1n", "'x'", "Symbol()"] {
            assert!(
                matches!(
                    Realm::default().eval(&format!(
                        "Iterator.prototype.{method}.call({receiver},()=>true)"
                    )),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{method}: {receiver}"
            );
        }
        check(&format!(
            "let closed=0,i={{get next(){{throw 7;}},get return(){{closed++;throw 8;}}}},caught=false;try{{Iterator.prototype.{method}.call(i,()=>true);}}catch(e){{caught=e===7;}}caught && closed===0"
        ));
    }
}

#[test]
fn direct_acquisition_is_eager_but_steps_and_callbacks_are_lazy_and_next_is_cached() {
    for method in ["map", "filter"] {
        check(&format!(
            "let gets=0,n=0,calls=0,seen=[],receiver,argc,callbackThis,i={{}};Object.defineProperty(i,Symbol.iterator,{{get(){{throw 7;}}}});Object.defineProperty(i,'next',{{configurable:true,get(){{gets++;return function(){{'use strict';receiver=this;argc=arguments.length;return {{done:n===3,value:n++}};}};}}}});let h=Iterator.prototype.{method}.call(i,function(value,index){{'use strict';callbackThis=this;calls++;seen.push(value,index,arguments.length);Object.defineProperty(i,'next',{{value:()=>{{throw 8;}}}});return true;}},{{valueOf(){{throw 9;}}}});let eager=gets===1 && n===0 && calls===0;let first=h.next(7),second=h.next(),third=h.next(),end=h.next();eager && !first.done && !second.done && !third.done && end.done && end.value===undefined && gets===1 && callbackThis===undefined && receiver===i && argc===0 && seen.join(',')==='0,0,2,1,1,2,2,2,2'"
        ));
        check(&format!(
            "let receiver;let h=[7].values().{method}(function(){{receiver=this;return true;}});h.next();receiver===globalThis"
        ));
    }
    check(
        "let i=[1,2,3].values(),h=i.map(value=>value*2);i.next();h.next().value===4 && h.next().value===6 && h.next().done",
    );
    check(
        "let i=[1,2,3].values(),h=i.filter(()=>true);i.next();h.next().value===2 && h.next().value===3 && h.next().done",
    );
}

#[test]
fn mapping_preserves_all_return_types_and_filtering_retains_original_values_with_source_indices() {
    for value in [
        "undefined",
        "null",
        "false",
        "true",
        "0",
        "-0",
        "NaN",
        "Infinity",
        "'x'",
        "Symbol()",
        "0n",
        "{}",
        "[]",
        "()=>0",
    ] {
        check(&format!(
            "let value={value},h=[7].values().map(()=>value),r=h.next();!r.done && Object.is(r.value,value) && h.next().done"
        ));
    }
    check(
        "let seen=[],h=[0,1,2,3,4].values().filter((value,index)=>{seen.push(index);return index%2;});h.next().value===1 && h.next().value===3 && h.next().done && seen.join(',')==='0,1,2,3,4'",
    );
    check(
        "let original={x:1},replacement={},slot=original,n=0;let h=Iterator.prototype.filter.call({next:()=>({done:n++>0,value:slot})},value=>{value.x=2;slot=replacement;return {valueOf(){throw 7;},toString(){throw 8;}};});h.next().value===original && original.x===2 && h.next().done && slot===replacement",
    );
    check(
        "let n=0,h=Iterator.prototype.filter.call({next:()=>({done:n===7,value:n++})},(value,index)=>[undefined,null,false,0,NaN,0n,''][index]);h.next().done",
    );
    check(
        "let h=[undefined].values().filter(()=>true),r=h.next();!r.done && r.value===undefined && h.next().done",
    );
}

#[test]
fn return_before_start_completes_before_cleanup_and_closes_without_stepping_or_callback_calls() {
    for method in ["map", "filter"] {
        check(&format!(
            "let gets=0,nexts=0,callbacks=0,closed=0,receiver,argc,reentered=false,h,i={{get next(){{gets++;return ()=>{{nexts++;throw 7;}};}},return:function(){{'use strict';receiver=this;argc=arguments.length;closed++;reentered=h.next().done && h.return().done;return {{get done(){{throw 8;}},get value(){{throw 9;}}}};}}}};h=Iterator.prototype.{method}.call(i,()=>{{callbacks++;}});let r=h.return(10);r.done && r.value===undefined && gets===1 && nexts===0 && callbacks===0 && closed===1 && receiver===i && argc===0 && reentered && h.next().done && h.return().done && closed===1"
        ));
        for ret in ["return 7;", "throw sentinel;"] {
            check(&format!(
                "let sentinel={{}},closed=0,h=Iterator.prototype.{method}.call({{next:()=>{{throw 7;}},return(){{closed++;{ret}}}}},()=>true),caught=false;try{{h.return();}}catch(e){{caught=e===sentinel || e instanceof TypeError;}}caught && closed===1 && h.next().done && h.return().done && closed===1"
            ));
        }
        check(&format!(
            "let nexts=0,h=Iterator.prototype.{method}.call({{next:()=>{{nexts++;throw 7;}}}},()=>true);h.return().done && h.next().done && nexts===0"
        ));
    }
}

#[test]
fn yielded_return_uses_live_cleanup_and_rejects_reentry_until_completion() {
    for method in ["map", "filter"] {
        check(&format!(
            "let old=0,closed=0,reentries=0,i={{next:()=>({{done:false,value:7}}),return(){{old++;throw 8;}}}},h=Iterator.prototype.{method}.call(i,()=>true);h.next();Object.defineProperty(i,'return',{{get(){{return ()=>{{closed++;try{{h.next();}}catch(e){{if(e instanceof TypeError)reentries++;}}try{{h.return();}}catch(e){{if(e instanceof TypeError)reentries++;}}return {{}};}};}}}});h.return().done && h.next().done && old===0 && closed===1 && reentries===2"
        ));
        for ret in ["return 7;", "throw sentinel;"] {
            check(&format!(
                "let sentinel={{}},closed=0,i={{next:()=>({{done:false,value:7}}),return(){{closed++;{ret}}}}},h=Iterator.prototype.{method}.call(i,()=>true);h.next();let caught=false;try{{h.return();}}catch(e){{caught=e===sentinel || e instanceof TypeError;}}caught && closed===1 && h.next().done && h.return().done && closed===1"
            ));
        }
        check(&format!(
            "let n=0,h=Iterator.prototype.{method}.call({{next:()=>({{done:n++>0,value:7}}),get return(){{throw 8;}}}},()=>true);h.next();h.next().done && h.return().done"
        ));
    }
    check(
        "let closed=0,source={next:()=>({done:false,value:7}),return(){closed++;return {};}};let h=Iterator.prototype.map.call(source,x=>x).filter(()=>true).map(x=>x);h.return().done && closed===1 && h.next().done",
    );
}

#[test]
fn callback_throws_close_once_preserve_incoming_identity_and_permanently_complete() {
    for method in ["map", "filter"] {
        for ret in ["return {};", "return 7;", "throw 8;"] {
            check(&format!(
                "let sentinel={{}},closed=0,calls=0,i={{next:()=>({{done:false,value:7}}),return(){{closed++;{ret}}}}},h=Iterator.prototype.{method}.call(i,()=>{{calls++;throw sentinel;}}),caught=false;try{{h.next();}}catch(e){{caught=e===sentinel;}}caught && closed===1 && calls===1 && h.next().done && h.return().done && closed===1"
            ));
        }
        check(&format!(
            "let sentinel={{}},closed=0,i={{next:()=>({{done:false,value:7}}),get return(){{closed++;throw 8;}}}},h=Iterator.prototype.{method}.call(i,()=>{{throw sentinel;}}),caught=false;try{{h.next();}}catch(e){{caught=e===sentinel;}}caught && closed===1 && h.next().done"
        ));
        check(&format!(
            "let closed=0,h=Iterator.prototype.{method}.call({{next:()=>({{done:false,value:7}}),return(){{closed++;return {{}};}}}},()=>h.next()),caught=false;try{{h.next();}}catch(e){{caught=e instanceof TypeError;}}caught && closed===1 && h.next().done"
        ));
        check(&format!(
            "let h,seen=0;h=[7].values().{method}(()=>{{try{{h.next();}}catch(e){{if(e instanceof TypeError)seen++;}}try{{h.return();}}catch(e){{if(e instanceof TypeError)seen++;}}return true;}});!h.next().done && seen===2 && h.next().done"
        ));
    }
}

#[test]
fn step_errors_skip_cleanup_and_done_skips_value_then_stays_completed() {
    for method in ["map", "filter"] {
        for next in [
            "()=>{throw 7;}",
            "()=>7",
            "()=>({get done(){throw 7;}})",
            "()=>({done:false,get value(){throw 7;}})",
            "7",
        ] {
            check(&format!(
                "let closed=0,calls=0,h=Iterator.prototype.{method}.call({{next:{next},return(){{closed++;throw 8;}}}},()=>{{calls++;}}),caught=false;try{{h.next();}}catch(e){{caught=e===7 || e instanceof TypeError;}}caught && closed===0 && calls===0 && h.next().done && h.return().done && closed===0"
            ));
        }
        check(&format!(
            "let h=Iterator.prototype.{method}.call({{next:()=>({{done:true,get value(){{throw 7;}}}}),get return(){{throw 8;}}}},()=>{{throw 9;}});h.next().done && h.return().done"
        ));
        check(&format!(
            "let h,closed=0;h=Iterator.prototype.{method}.call({{next:()=>h.next(),return(){{closed++;return {{}};}}}},()=>true);let caught=false;try{{h.next();}}catch(e){{caught=e instanceof TypeError;}}caught && closed===0 && h.next().done"
        ));
    }
}

#[test]
fn captures_survive_collection_and_are_released_on_completion_and_intrinsics_stay_rooted() {
    for method in ["map", "filter"] {
        let mut realm = Realm::default();
        let Value::Object(source)=realm.eval(&format!("let i={{next:()=>({{done:false,value:7}})}},callback=()=>true,h=Iterator.prototype.{method}.call(i,callback);i")).unwrap() else {panic!("source")};
        let Value::Object(next) = realm.eval("i.next").unwrap() else {
            panic!("next")
        };
        let Value::Object(callback) = realm.eval("callback").unwrap() else {
            panic!("callback")
        };
        realm.eval("delete i.next;i=null;callback=null;").unwrap();
        realm.collect(usize::MAX).unwrap();
        assert!(
            realm.inspect_object(&source).is_ok()
                && realm.inspect_object(&next).is_ok()
                && realm.inspect_object(&callback).is_ok()
        );
        assert_eq!(realm.eval("h.next().done"), Ok(Value::Boolean(false)));
        realm.collect(usize::MAX).unwrap();
        assert_eq!(realm.eval("h.next().done"), Ok(Value::Boolean(false)));
        realm.eval("h.return()").unwrap();
        assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 1);
        assert!(
            realm.inspect_object(&source).is_err()
                && realm.inspect_object(&next).is_err()
                && realm.inspect_object(&callback).is_err()
        );
        realm
            .eval(&format!(
                "h=null;let f=Iterator.prototype.{method};delete Iterator.prototype.{method};"
            ))
            .unwrap();
        assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
        assert_eq!(
            realm.eval("f.call([7].values(),()=>true).next().done"),
            Ok(Value::Boolean(false))
        );
    }
}

#[test]
fn metadata_and_shared_helper_brand_are_standard() {
    for method in ["map", "filter"] {
        check(&format!(
            "let f=Iterator.prototype.{method},d=Object.getOwnPropertyDescriptor(Iterator.prototype,'{method}'),h=[7].values().{method}(()=>true),p=Object.getPrototypeOf(h),c=Iterator.concat([8]);f.name==='{method}' && f.length===1 && Object.getPrototypeOf(f)===Function.prototype && !Object.hasOwn(f,'prototype') && d.value===f && d.writable && !d.enumerable && d.configurable && h instanceof Iterator && Object.getPrototypeOf(p)===Iterator.prototype && Object.prototype.toString.call(h)==='[object Iterator Helper]' && h.next===c.next && h.return===c.return && h[Symbol.iterator]()===h"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new Iterator.prototype.{method}(()=>true)")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        check(&format!(
            "let h=[7].values().{method}(()=>true),wrong=Object.create(h),caught=false;try{{h.next.call(wrong);}}catch(e){{caught=e instanceof TypeError;}}Object.setPrototypeOf(h,null);caught && Iterator.concat().next.call(h).done===false"
        ));
    }
}

#[test]
fn large_default_pipelines_and_filter_skips_work_while_host_aborts_complete_without_cleanup() {
    check(
        "let n=0,h=Iterator.prototype.map.call({next:()=>({done:n===10000,value:n++})},(value,index)=>value+index).filter(value=>value%4===0),a=h.toArray();a.length===5000 && a[0]===0 && a[4999]===19996",
    );
    check(
        "let n=0,h=Iterator.prototype.filter.call({next:()=>({done:n===10000,value:n++})},()=>false);h.next().done && n===10001",
    );
    for method in ["map", "filter"] {
        let mut realm = Realm::new(Limits {
            max_steps: Some(5000),
            ..Limits::default()
        });
        let next = if method == "map" {
            "()=>{while(true){};}"
        } else {
            "()=>({done:false,value:7})"
        };
        realm.eval(&format!("let flag=0,h=Iterator.prototype.{method}.call({{next:{next},return:()=>{{flag=3;return {{}};}}}},()=>false);")).unwrap();
        assert!(matches!(
            realm.eval("try{h.next();}catch{flag=1;}finally{flag=2;}"),
            Err(Error::Limit { .. })
        ));
        assert_eq!(
            realm.eval("flag===0 && h.next().done && h.return().done"),
            Ok(Value::Boolean(true))
        );
        let mut realm = Realm::default();
        realm.eval(&format!("let flag=0,h=Iterator.prototype.{method}.call({{next:()=>({{done:false,value:7}}),return:()=>{{flag=3;return {{}};}}}},()=>Function('function* gap(){{}}'));")).unwrap();
        assert!(matches!(
            realm.eval("try{h.next();}catch{flag=1;}finally{flag=2;}"),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(
            realm.eval("flag===0 && h.next().done && h.return().done"),
            Ok(Value::Boolean(true))
        );
    }
}
