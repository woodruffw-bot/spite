//! Published-edition take/drop conversion, countdown, and closing (27.1.3.3.2/11).

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
fn receiver_validation_precedes_conversion_and_conversion_precedes_eager_next_capture() {
    for method in ["take", "drop"] {
        check(&format!(
            "let effects=[],n=0,i={{get next(){{effects.push('next');return ()=>{{n++;return {{done:true}};}};}}}},limit={{valueOf(){{effects.push('limit');return 0;}}}},h=Iterator.prototype.{method}.call(i,limit,{{valueOf(){{throw 7;}}}});effects.join(',')==='limit,next' && n===0 && h instanceof Iterator"
        ));
        for receiver in ["undefined", "null", "true", "1", "1n", "'x'", "Symbol()"] {
            assert!(
                matches!(
                    Realm::default().eval(&format!(
                        "Iterator.prototype.{method}.call({receiver},{{valueOf(){{throw 7;}}}})"
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
            "let effects=[],limit={{[Symbol.toPrimitive](hint){{effects.push(hint);return '1.9';}}}},i={{get next(){{effects.push('next');return ()=>({{done:true}});}}}};Iterator.prototype.{method}.call(i,limit);effects.join(',')==='number,next'"
        ));
    }
}

#[test]
fn conversion_validation_errors_close_before_next_and_keep_incoming_error_identity() {
    for method in ["take", "drop"] {
        for limit in ["undefined", "NaN", "'x'", "-1", "-1.5", "-Infinity"] {
            for ret in ["return {};", "return 7;", "throw 8;"] {
                check(&format!(
                    "let closed=0,i={{get next(){{throw 9;}},return(){{closed++;{ret}}}}},caught=false;try{{Iterator.prototype.{method}.call(i,{limit});}}catch(e){{caught=e instanceof RangeError;}}caught && closed===1"
                ));
            }
        }
        for limit in ["1n", "Symbol()"] {
            check(&format!(
                "let closed=0,i={{get next(){{throw 7;}},return(){{closed++;throw 8;}}}},caught=false;try{{Iterator.prototype.{method}.call(i,{limit});}}catch(e){{caught=e instanceof TypeError;}}caught && closed===1"
            ));
        }
        check(&format!(
            "let sentinel={{}},closed=0,i={{get next(){{throw 7;}},get return(){{closed++;throw 8;}}}},caught=false;try{{Iterator.prototype.{method}.call(i,{{valueOf(){{throw sentinel;}}}});}}catch(e){{caught=e===sentinel;}}caught && closed===1"
        ));
        check(&format!(
            "let closed=0,i={{get next(){{throw 7;}},return(){{closed++;return {{}};}}}},caught=false;try{{Iterator.prototype.{method}.call(i,0);}}catch(e){{caught=e===7;}}caught && closed===0"
        ));
    }
}

#[test]
fn finite_counts_truncate_and_large_counts_and_infinity_follow_the_published_baseline() {
    for limit in ["0", "-0", "-0.5", "null", "false", "''", "[]"] {
        check(&format!(
            "[1,2].values().take({limit}).toArray().length===0 && [1,2].values().drop({limit}).toArray().join(',')==='1,2'"
        ));
    }
    for limit in [
        "1",
        "1.9",
        "true",
        "'1.9'",
        "[1]",
        "{valueOf(){return {};},toString(){return '1';}}",
    ] {
        check(&format!(
            "[1,2,3].values().take({limit}).toArray().join(',')==='1' && [1,2,3].values().drop({limit}).toArray().join(',')==='2,3'"
        ));
    }
    // Edition 17 imposes no MAX_SAFE_INTEGER check. Later pinned upstream
    // assertions for that restriction remain outside the selected corpus.
    for limit in [
        "Number.MAX_SAFE_INTEGER+1",
        "18446744073709551616",
        "Number.MAX_VALUE",
        "Infinity",
    ] {
        check(&format!(
            "[1,2,3].values().take({limit}).toArray().join(',')==='1,2,3' && [1,2,3].values().drop({limit}).next().done"
        ));
    }
}

#[test]
fn cached_direct_next_uses_zero_arguments_and_drop_never_reads_discarded_values() {
    check(
        "let gets=0,n=0,receiver,argc,i={get next(){gets++;return function(){'use strict';receiver=this;argc=arguments.length;return {done:false,value:n++};};}},h=Iterator.prototype.take.call(i,2);Object.defineProperty(i,'next',{value:()=>{throw 7;}});Object.defineProperty(i,Symbol.iterator,{get(){throw 8;}});h.next(9).value===0 && h.next().value===1 && h.next().done && gets===1 && n===2 && receiver===i && argc===0",
    );
    check(
        "let gets=0,n=0,values=0,receiver,argc,i={get next(){gets++;return function(){'use strict';receiver=this;argc=arguments.length;let value=n++;return {done:value===4,get value(){values++;if(value<2)throw 7;return value;}};};}},h=Iterator.prototype.drop.call(i,2);Object.defineProperty(i,'next',{value:()=>{throw 8;}});Object.defineProperty(i,Symbol.iterator,{get(){throw 9;}});h.next().value===2 && h.next().value===3 && h.next().done && gets===1 && values===2 && receiver===i && argc===0",
    );
    check(
        "let n=0,values=0,h=Iterator.prototype.drop.call({next:()=>({done:n++===3,get value(){values++;throw 7;}})},Infinity);h.next().done && values===0",
    );
    check(
        "let source=[0,1,2,3].values(),h=source.drop(1);source.next();h.next().value===2 && h.next().value===3 && h.next().done",
    );
    check(
        "let source=[0,1,2,3].values(),h=source.take(1);source.next();h.next().value===1 && h.next().done && source.next().value===2",
    );
}

#[test]
fn take_closes_exactly_when_count_is_exhausted_and_drop_only_closes_on_return() {
    check(
        "let closed=0,n=0,i={next:()=>({done:false,value:n++}),return(){closed++;return {};}},h=Iterator.prototype.take.call(i,2);let a=h.next(),b=h.next(),before=closed;let end=h.next();a.value===0 && b.value===1 && before===0 && end.done && end.value===undefined && closed===1 && n===2 && h.next().done && h.return().done && closed===1",
    );
    check(
        "let closed=0,n=0,h=Iterator.prototype.take.call({next:()=>{n++;throw 7;},return(){closed++;return {};}},0);h.next().done && closed===1 && n===0 && h.return().done && closed===1",
    );
    check(
        "let n=0,h=Iterator.prototype.take.call({next:()=>({done:n++===1,value:7}),get return(){throw 8;}},2);h.next().value===7 && h.next().done && h.return().done",
    );
    check(
        "let n=0,h=Iterator.prototype.drop.call({next:()=>({done:n++===1,get value(){throw 7;}}),get return(){throw 8;}},2);h.next().done && h.return().done",
    );
    check(
        "let closed=0,h=Iterator.prototype.drop.call({next:()=>({done:false,value:7}),return(){closed++;return {};}},1);h.next().value===7 && closed===0 && h.return().done && closed===1 && h.next().done",
    );
    check("let h=Iterator.prototype.take.call({next:7},0);h.next().done");
    for ret in ["return 7;", "throw sentinel;"] {
        check(&format!(
            "let sentinel={{}},closed=0,h=Iterator.prototype.take.call({{next:()=>{{throw 8;}},return(){{closed++;{ret}}}}},0),caught=false;try{{h.next();}}catch(e){{caught=e===sentinel || e instanceof TypeError;}}caught && closed===1 && h.next().done && h.return().done && closed===1"
        ));
    }
}

#[test]
fn early_return_completes_before_cleanup_yielded_closing_rejects_reentry_and_step_errors_skip_close()
 {
    for method in ["take", "drop"] {
        check(&format!(
            "let nexts=0,closed=0,reentered=false,h=Iterator.prototype.{method}.call({{next:()=>{{nexts++;throw 7;}},return(){{closed++;reentered=h.next().done && h.return().done;return {{}};}}}},1);h.return(8).done && nexts===0 && closed===1 && reentered && h.next().done"
        ));
        for next in [
            "()=>{throw 7;}",
            "()=>7",
            "()=>({get done(){throw 7;}})",
            "7",
        ] {
            check(&format!(
                "let closed=0,h=Iterator.prototype.{method}.call({{next:{next},return(){{closed++;throw 8;}}}},1),caught=false;try{{h.next();}}catch(e){{caught=e===7 || e instanceof TypeError;}}caught && closed===0 && h.next().done && h.return().done"
            ));
        }
        check(&format!(
            "let closed=0,h=Iterator.prototype.{method}.call({{next:()=>({{done:false,get value(){{throw 7;}}}}),return(){{closed++;throw 8;}}}},0.5+1),caught=false;try{{h.next();}}catch(e){{caught=e===7;}}caught && closed===0 && h.next().done"
        ));
        check(&format!(
            "let h,closed=0;h=Iterator.prototype.{method}.call({{next:()=>h.next(),return(){{closed++;return {{}};}}}},1);let caught=false;try{{h.next();}}catch(e){{caught=e instanceof TypeError;}}caught && closed===0 && h.next().done"
        ));
    }
    check(
        "let h,reentries=0;h=Iterator.prototype.take.call({next:()=>{throw 7;},return(){try{h.next();}catch(e){if(e instanceof TypeError)reentries++;}try{h.return();}catch(e){if(e instanceof TypeError)reentries++;}return {};}},0);h.next().done && reentries===2",
    );
    check(
        "let closed=0,h=Iterator.prototype.take.call({next:()=>({done:false,value:7}),return(){closed++;return {}; }},1).drop(0).map(value=>value+1).take(1);h.toArray().join(',')==='8' && closed===1",
    );
}

#[test]
fn metadata_helper_identity_and_intrinsic_retention_are_standard() {
    for method in ["take", "drop"] {
        check(&format!(
            "let f=Iterator.prototype.{method},d=Object.getOwnPropertyDescriptor(Iterator.prototype,'{method}'),h=[7].values().{method}(0),c=Iterator.concat();f.name==='{method}' && f.length===1 && Object.getPrototypeOf(f)===Function.prototype && !Object.hasOwn(f,'prototype') && d.value===f && d.writable && !d.enumerable && d.configurable && h instanceof Iterator && h.next===c.next && h.return===c.return && Object.prototype.toString.call(h)==='[object Iterator Helper]'"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new Iterator.prototype.{method}(1)")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        let mut realm = Realm::default();
        realm
            .eval(&format!(
                "let f=Iterator.prototype.{method};delete Iterator.prototype.{method};"
            ))
            .unwrap();
        assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
        assert_eq!(
            realm.eval("f.call([7,8].values(),1).next().value"),
            Ok(Value::Number(if method == "take" { 7.0 } else { 8.0 }))
        );
    }
}

#[test]
fn captured_sources_and_cached_next_survive_collection_and_release_after_return() {
    for method in ["take", "drop"] {
        let mut realm = Realm::default();
        let Value::Object(source)=realm.eval(&format!("let i={{next:()=>({{done:false,value:7}})}},h=Iterator.prototype.{method}.call(i,Infinity);i")).unwrap() else {panic!("source")};
        let Value::Object(next) = realm.eval("i.next").unwrap() else {
            panic!("next")
        };
        realm.eval("delete i.next;i=null;").unwrap();
        realm.collect(usize::MAX).unwrap();
        assert!(realm.inspect_object(&source).is_ok() && realm.inspect_object(&next).is_ok());
        if method == "take" {
            assert_eq!(realm.eval("h.next().value"), Ok(Value::Number(7.0)));
        }
        realm.eval("h.return()").unwrap();
        assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 1);
        assert!(realm.inspect_object(&source).is_err() && realm.inspect_object(&next).is_err());
        realm.eval("h=null").unwrap();
        assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    }
}

#[test]
fn large_default_inputs_and_internal_counts_work_without_bigint_value_quotas() {
    check(
        "let n=0,i=Iterator.prototype.take.call({next:()=>({done:false,value:n++})},10000),a=i.toArray();a.length===10000 && a[9999]===9999 && n===10000",
    );
    check(
        "let n=0,h=Iterator.prototype.drop.call({next:()=>({done:n===10001,value:n++})},10000);h.next().value===10000 && h.next().done",
    );
    let mut realm = Realm::new(Limits {
        max_bigint_bits: Some(0),
        ..Limits::default()
    });
    assert_eq!(realm.eval("[7].values().take(18446744073709551616).next().value===7 && [7].values().drop(Number.MAX_VALUE).next().done"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_work_and_unsupported_failures_complete_resumes_without_javascript_cleanup() {
    for method in ["take", "drop"] {
        let mut realm = Realm::new(Limits {
            max_steps: Some(5000),
            ..Limits::default()
        });
        let next = if method == "take" {
            "()=>{while(true){};}"
        } else {
            "()=>({done:false,value:7})"
        };
        realm.eval(&format!("let flag=0,h=Iterator.prototype.{method}.call({{next:{next},return:()=>{{flag=3;return {{}};}}}},Infinity);")).unwrap();
        assert!(matches!(
            realm.eval("try{h.next();}catch{flag=1;}finally{flag=2;}"),
            Err(Error::Limit { .. })
        ));
        assert_eq!(
            realm.eval("flag===0 && h.next().done && h.return().done"),
            Ok(Value::Boolean(true))
        );
        let mut realm = Realm::default();
        realm.eval(&format!("let flag=0,h=Iterator.prototype.{method}.call({{next:()=>Function('class C{{field;}}'),return:()=>{{flag=3;return {{}};}}}},1);")).unwrap();
        assert!(matches!(
            realm.eval("try{h.next();}catch{flag=1;}finally{flag=2;}"),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(
            realm.eval("flag===0 && h.next().done && h.return().done"),
            Ok(Value::Boolean(true))
        );
        let mut realm = Realm::default();
        realm
            .eval("let flag=0,i={get next(){flag=4;throw 7;},return(){flag=3;return {};}}")
            .unwrap();
        assert!(matches!(realm.eval(&format!("try{{Iterator.prototype.{method}.call(i,{{valueOf:()=>Function('class C{{field;}}')}});}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Unsupported{..})));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
