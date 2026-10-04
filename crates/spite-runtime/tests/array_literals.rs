//! Array literal accumulation, observable element order, and holes (13.2.4).

mod common;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn holes_and_trailing_commas_preserve_presence_and_length() {
    check(
        "[].length===0 && [,].length===1 && [,,].length===2 && [1,].length===1 && [1,,].length===2",
    );
    check(
        "let a=[,undefined,,3,];a.length===4 && !Object.hasOwn(a,'0') && Object.hasOwn(a,'1') && !Object.hasOwn(a,'2') && Object.hasOwn(a,'3') && a[0]===undefined && a[3]===3",
    );
    check(
        "let a=[1,,];Object.defineProperty(Array.prototype,'1',{value:9,writable:true,configurable:true});a.length===2 && a[1]===9 && !Object.hasOwn(a,'1')",
    );
    check(
        "let a=[1,[2],{}];Array.isArray(a) && Array.isArray(a[1]) && a[1][0]===2 && !Array.isArray(a[2])",
    );
}

#[test]
fn literals_use_intrinsics_independently_of_global_array_binding() {
    check(
        "let A=Array;Array=()=>{throw 1;};let a=[1];a[0]===1 && a instanceof A && Object.getPrototypeOf(a)===A.prototype",
    );
    check(
        "let A=Array;delete globalThis.Array;let a=[,2,];a.length===2 && a[1]===2 && A.isArray(a) && Object.getPrototypeOf(a)===A.prototype",
    );
    check("function f(){let Array=7;return [3];}f()[0]===3");
}

#[test]
fn element_evaluation_is_left_to_right_and_stops_on_throw() {
    check(
        "let n=0,a=[++n,,n+=2,(n++,n),];n===4 && a.length===4 && a[0]===1 && a[2]===3 && a[3]===4",
    );
    let mut realm = Realm::default();
    realm.eval("let n=0;function fail(){throw 7;}").unwrap();
    assert_eq!(
        realm.eval("[++n,,fail(),++n]"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(realm.eval("n"), Ok(Value::Number(1.0)));
    check("let a=[0];a=[a[0]+1,a];a[0]===1 && a[1][0]===0 && a!==a[1]");
}

#[test]
fn elements_define_own_properties_bypassing_inherited_setters() {
    check(
        "let n=0;Object.defineProperty(Array.prototype,'0',{set:()=>{n++;},configurable:true});let a=[4];a[0]===4 && Object.hasOwn(a,'0') && n===0",
    );
    check(
        "Object.defineProperty(Array.prototype,'0',{value:9,writable:false,configurable:true});Object.defineProperty(Array.prototype,'length',{writable:false});let a=[1,,];a[0]===1 && a.length===2 && !Object.hasOwn(a,'1')",
    );
    check(
        "let a=[Object.defineProperty(Array.prototype,'1',{value:9,writable:false,configurable:true}),2];a[1]===2",
    );
}

#[test]
fn literal_elements_do_not_infer_anonymous_function_names() {
    check(
        "let a=[function(){},()=>0,function named(){}];a[0].name==='' && a[1].name==='' && a[2].name==='named'",
    );
    check("let f;let a=[f=()=>1];a[0].name==='f'");
}

#[test]
fn arrays_work_in_member_call_new_and_function_contexts() {
    check("[function(x){return this[1]+x;},3][0](4)===7");
    check("function F(){this.x=7;}new [F][0]().x===7");
    assert!(matches!(
        Realm::default().eval("new []"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    check("function f(a=[2]){return a;}f()!==f() && f()[0]===2");
    check(
        "let a=[0],out=[];for(let i=0;i<3;i++){out[i]=()=>i;}out[0]()===0 && out[2]()===2 && out.length===3",
    );
    check("let ok=false;for(let a=['x' in {x:1}];a[0];){ok=true;break;}ok");
}

#[test]
fn arrays_trace_nested_values_and_allow_sparse_holes_under_property_limits() {
    let mut realm = Realm::default();
    realm.eval("let a=[{x:1},[{y:2}]]").unwrap();
    realm.collect(10_000).unwrap();
    assert_eq!(
        realm.eval("a[0].x===1 && a[1][0].y===2"),
        Ok(Value::Boolean(true))
    );
    let holes = ",".repeat(2000);
    assert_eq!(
        realm.eval(&format!("[{holes}].length")),
        Ok(Value::Number(2000.0))
    );
}

#[test]
fn allocation_precedes_elements_and_literals_do_not_use_argument_limits() {
    let mut realm = Realm::new(Limits {
        max_heap_entries: common::REALM_ENTRIES,
        ..Limits::default()
    });
    realm.eval("let n=0,flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{[n++];}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("n===0 && flag===0"), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_arguments: 0,
        ..Limits::default()
    });
    assert_eq!(realm.eval("[1,2,3].length"), Ok(Value::Number(3.0)));
}
