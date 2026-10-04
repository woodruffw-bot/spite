//! Direct predicate consumers and normal/abrupt closing (27.1.3.3.3/5/10).

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

const METHODS: [(&str, &str, &str, &str); 3] = [
    ("every", "false", "false", "true"),
    ("some", "true", "true", "false"),
    ("find", "true", "7", "undefined"),
];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn invalid_callbacks_close_before_next_but_primitive_receivers_are_rejected_first() {
    for (method, _, _, _) in METHODS {
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
        check(&format!(
            "let closed=0,i={{get next(){{throw 7;}},get return(){{closed++;throw 8;}}}},caught=false;try{{Iterator.prototype.{method}.call(i);}}catch(e){{caught=e instanceof TypeError;}}caught && closed===1"
        ));
        for receiver in ["undefined", "null", "true", "1", "1n", "'x'", "Symbol()"] {
            assert!(
                matches!(
                    Realm::default().eval(&format!(
                        "Iterator.prototype.{method}.call({receiver},()=>0)"
                    )),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{method}: {receiver}"
            );
        }
    }
}

#[test]
fn short_circuit_closes_once_with_live_return_and_original_receiver_without_extra_steps() {
    for (method, predicate, expected, _) in METHODS {
        check(&format!(
            "let gets=0,calls=0,closed=0,argc,receiver,old=0,i={{next(){{calls++;return {{done:false,value:7}};}},return(){{old++;throw 8;}}}};let result=Iterator.prototype.{method}.call(i,()=>{{Object.defineProperty(i,'return',{{get(){{gets++;return function(){{'use strict';closed++;argc=arguments.length;receiver=this;return {{get value(){{throw 9;}},get done(){{throw 10;}}}};}};}}}});return {predicate};}});result==={expected} && gets===1 && calls===1 && closed===1 && receiver===i && argc===0 && old===0"
        ));
        for ret in ["undefined", "null"] {
            check(&format!(
                "let calls=0,i={{next:()=>({{done:false,value:7}}),return:{ret}}};Iterator.prototype.{method}.call(i,()=>{{calls++;return {predicate};}})==={expected} && calls===1"
            ));
        }
    }
    check(
        "let a=[1,2,3].values(),b=[1,2,3].values(),c=[1,2,3].values();a.every(value=>value<2)===false && a.next().value===3 && b.some(value=>value===2)===true && b.next().value===3 && c.find(value=>value===2)===2 && c.next().value===3",
    );
    check("let i=Iterator.concat([1,2],[3]);i.some(value=>value===1) && i.next().done");
}

#[test]
fn normal_close_errors_replace_results_while_callback_throws_keep_incoming_identity() {
    for (method, predicate, _, _) in METHODS {
        for ret in ["return 7;", "return null;", "return undefined;"] {
            check(&format!(
                "let closed=0,i={{next:()=>({{done:false,value:7}}),return(){{closed++;{ret}}}}},caught=false;try{{Iterator.prototype.{method}.call(i,()=>{predicate});}}catch(e){{caught=e instanceof TypeError;}}caught && closed===1"
            ));
        }
        for ret in ["return 7;", "throw 8;", "return {};"] {
            check(&format!(
                "let sentinel={{}},closed=0,calls=0,i={{next:()=>({{done:false,value:7}}),return(){{closed++;{ret}}}}},caught=false;try{{Iterator.prototype.{method}.call(i,()=>{{calls++;throw sentinel;}});}}catch(e){{caught=e===sentinel;}}caught && closed===1 && calls===1"
            ));
        }
        check(&format!(
            "let sentinel={{}},i={{next:()=>({{done:false,value:7}}),get return(){{throw sentinel;}}}},caught=false;try{{Iterator.prototype.{method}.call(i,()=>{predicate});}}catch(e){{caught=e===sentinel;}}caught"
        ));
        check(&format!(
            "let sentinel={{}},i={{next:()=>({{done:false,value:7}}),return(){{throw sentinel;}}}},caught=false;try{{Iterator.prototype.{method}.call(i,()=>{predicate});}}catch(e){{caught=e===sentinel;}}caught"
        ));
        check(&format!(
            "let sentinel={{}},closed=0,i={{next:()=>({{done:false,value:7}}),get return(){{closed++;throw 8;}}}},caught=false;try{{Iterator.prototype.{method}.call(i,()=>{{throw sentinel;}});}}catch(e){{caught=e===sentinel;}}caught && closed===1"
        ));
        check(&format!(
            "let i={{next:()=>({{done:false,value:7}}),return:7}},caught=false;try{{Iterator.prototype.{method}.call(i,()=>{predicate});}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
}

#[test]
fn traversal_caches_next_and_calls_predicates_with_undefined_this_value_and_exact_indices() {
    for (method, _, _, empty) in METHODS {
        let continuing = if method == "every" { "true" } else { "false" };
        check(&format!(
            "let gets=0,n=0,seen=[],receiver,argc,callbackThis,i={{}};Object.defineProperty(i,Symbol.iterator,{{get(){{throw 7;}}}});Object.defineProperty(i,'next',{{configurable:true,get(){{gets++;return function(){{'use strict';receiver=this;argc=arguments.length;return {{done:n===3,value:n++}};}};}}}});let result=Iterator.prototype.{method}.call(i,function(value,index){{'use strict';callbackThis=this;seen.push(value,index,arguments.length);Object.defineProperty(i,'next',{{value:()=>{{throw 8;}}}});return {continuing};}},{{valueOf(){{throw 9;}}}});result==={empty} && gets===1 && callbackThis===undefined && receiver===i && argc===0 && seen.join(',')==='0,0,2,1,1,2,2,2,2'"
        ));
        check(&format!(
            "let receiver,n=0;Iterator.prototype.{method}.call({{next:()=>({{done:n++>0,value:7}})}},function(){{receiver=this;return {continuing};}});receiver===globalThis"
        ));
    }
    check(
        "let original={x:1},replacement={},slot=original,i={next:()=>({done:false,value:slot}),return:()=>({})};let result=Iterator.prototype.find.call(i,value=>{value.x=2;slot=replacement;return true;});result===original && result.x===2 && slot===replacement",
    );
    check(
        "let count=0,i={next:()=>({done:false,get value(){count++;return undefined;}}),return:()=>({})};Iterator.prototype.find.call(i,()=>true)===undefined && count===1",
    );
}

#[test]
fn truthiness_does_not_coerce_objects_and_handles_all_primitive_categories() {
    for result in [
        "undefined",
        "null",
        "false",
        "0",
        "-0",
        "NaN",
        "0n",
        "''",
        "true",
        "1",
        "-1",
        "1n",
        "'x'",
        "Symbol()",
        "{valueOf(){throw 8;},toString(){throw 9;}}",
    ] {
        for (method, _, _, _) in METHODS {
            check(&format!(
                "let result={result},calls=0,closed=0,i={{next(){{return {{value:7,done:calls++===1}};}},return(){{closed++;return {{}};}}}};let actual=Iterator.prototype.{method}.call(i,()=>result);let expected=('{method}'==='every') ? !!result : ('{method}'==='some' ? !!result : (result ? 7 : undefined));let expectedClose=('{method}'==='every') ? !result : !!result;actual===expected && closed===Number(expectedClose)"
            ));
        }
    }
}

#[test]
fn acquisition_and_step_errors_do_not_close_and_exhaustion_never_reads_return_or_value() {
    for (method, _, _, empty) in METHODS {
        for next in [
            "()=>{throw 7;}",
            "()=>7",
            "()=>({get done(){throw 7;}})",
            "()=>({done:false,get value(){throw 7;}})",
            "7",
        ] {
            check(&format!(
                "let closed=0,calls=0,i={{next:{next},return(){{closed++;throw 8;}}}},caught=false;try{{Iterator.prototype.{method}.call(i,()=>{{calls++;}});}}catch(e){{caught=e===7 || e instanceof TypeError;}}caught && closed===0 && calls===0"
            ));
        }
        check(&format!(
            "let closed=0,i={{get next(){{throw 7;}},return(){{closed++;return {{}};}}}},caught=false;try{{Iterator.prototype.{method}.call(i,()=>true);}}catch(e){{caught=e===7;}}caught && closed===0"
        ));
        check(&format!(
            "let i={{next:()=>({{done:true,get value(){{throw 7;}}}}),get return(){{throw 8;}}}};Iterator.prototype.{method}.call(i,()=>{{throw 9;}})==={empty}"
        ));
    }
}

#[test]
fn standard_metadata_non_constructibility_and_intrinsic_retention_survive_collection() {
    for (method, _, _, _) in METHODS {
        check(&format!(
            "let f=Iterator.prototype.{method},d=Object.getOwnPropertyDescriptor(Iterator.prototype,'{method}');f.name==='{method}' && f.length===1 && Object.getPrototypeOf(f)===Function.prototype && !Object.hasOwn(f,'prototype') && d.value===f && d.writable && !d.enumerable && d.configurable"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new Iterator.prototype.{method}(()=>true)")),
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
            realm.eval("f.call([7].values(),()=>true)"),
            Ok(if method == "find" {
                Value::Number(7.0)
            } else {
                Value::Boolean(true)
            })
        );
    }
}

#[test]
fn large_default_inputs_work_and_host_aborts_skip_handlers_and_cleanup() {
    for (method, predicate, _, empty) in METHODS {
        let continuing = if method == "every" { "true" } else { "false" };
        check(&format!(
            "let n=0,sum=0;let result=Iterator.prototype.{method}.call({{next:()=>({{done:n===10000,value:n++}})}},(value,index)=>{{sum+=index;return {continuing};}});result==={empty} && sum===49995000"
        ));
        let mut realm = Realm::new(Limits {
            max_steps: Some(5000),
            ..Limits::default()
        });
        realm
            .eval("let flag=0,i={next:()=>({done:false,value:7}),return:()=>{flag=3;return {};}};")
            .unwrap();
        assert!(matches!(realm.eval(&format!("try{{Iterator.prototype.{method}.call(i,()=>{continuing});}}catch{{flag=1;}}finally{{flag=2;}}")),Err(Error::Limit{..})));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        for body in [
            format!("Iterator.prototype.{method}.call(i,Function)"),
            format!("i.return=Function;Iterator.prototype.{method}.call(i,()=>{predicate})"),
        ] {
            let mut realm = Realm::default();
            realm.eval("let flag=0,i={next:()=>({done:false,value:7}),return:()=>{flag=3;return {};}};").unwrap();
            assert!(matches!(
                realm.eval(&format!("try{{{body};}}catch{{flag=1;}}finally{{flag=2;}}")),
                Err(Error::Unsupported { .. })
            ));
            assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        }
    }
}
