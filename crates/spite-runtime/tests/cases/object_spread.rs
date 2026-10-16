//! CopyDataProperties for object initializer spread (13.2.5.5, 7.3.25).

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn primitive_sources_box_without_coercion_and_nullish_sources_are_skipped() {
    check(
        "let a={...null,...undefined,...true,...7,...7n,...Symbol()};Object.keys(a).length===0 && Object.getPrototypeOf(a)===Object.prototype",
    );
    check(
        "let a={...'A\\uD834\\uDF06\\uD800'};Object.keys(a).join(',')==='0,1,2,3' && a[0]==='A' && a[1]==='\\uD834' && a[2]==='\\uDF06' && a[3]==='\\uD800' && !Object.hasOwn(a,'length')",
    );
    check(
        "let calls=0,s=Symbol.toPrimitive,source={[s](){calls++;throw 7;},valueOf(){calls++;throw 8;}};let a={...source};a[s]===source[s] && a.valueOf===source.valueOf && calls===0",
    );
}

#[test]
fn only_enumerable_own_properties_are_copied_as_fresh_data() {
    check(
        "let source=Object.create({inherited:7});source.own=8;Object.defineProperty(source,'hidden',{get(){throw 9;}});let a={...source};a.own===8 && !Object.hasOwn(a,'inherited') && !Object.hasOwn(a,'hidden')",
    );
    check(
        "let source=Object.freeze({x:1});let a={...source},d=Object.getOwnPropertyDescriptor(a,'x');a!==source && d.value===1 && d.writable && d.enumerable && d.configurable && Object.isExtensible(a)",
    );
    check(
        "let source={set x(v){throw 7;}};let a={...source},d=Object.getOwnPropertyDescriptor(a,'x');d.value===undefined && d.writable && !Object.hasOwn(d,'set')",
    );
    check(
        "let source=[,7];source.x=8;let a={...source};a[1]===7 && a.x===8 && !Object.hasOwn(a,'0') && !Object.hasOwn(a,'length') && !Array.isArray(a)",
    );
}

#[test]
fn getter_receivers_and_key_order_include_symbols_after_strings() {
    check(
        "let log='',s=Symbol(),source={get 2(){log+='2';return 2;},get 1(){log+='1';return 1;},get z(){if(this!==source)throw 8;log+='z';return 3;},get a(){log+='a';return 4;},get [s](){log+='s';return 5;}};let a={...source};log==='12zas' && Object.getOwnPropertyNames(a).join(',')==='1,2,z,a' && Object.getOwnPropertySymbols(a)[0]===s && a[s]===5",
    );
    check(
        "let s=Symbol(),source={};Object.defineProperty(source,s,{get(){return 7;},enumerable:true});let a={...source},d=Object.getOwnPropertyDescriptor(a,s);d.value===7 && d.writable && d.enumerable && d.configurable && !Object.hasOwn(d,'get')",
    );
}

#[test]
fn keys_are_snapshotted_but_descriptors_and_values_are_live() {
    check(
        "let source={get a(){delete this.b;this.e=7;Object.defineProperty(this,'c',{enumerable:true});this.d=8;return 1;},b:2,e:3};Object.defineProperty(source,'c',{value:6,enumerable:false,configurable:true});let a={...source};Object.keys(a).join(',')==='a,e,c' && a.a===1 && a.e===7 && a.c===6 && !Object.hasOwn(a,'b') && !Object.hasOwn(a,'d')",
    );
    check(
        "let gets=0,source={get a(){Object.defineProperty(this,'b',{enumerable:false});return 1;},get b(){gets++;throw 8;}};let a={...source};a.a===1 && !Object.hasOwn(a,'b') && gets===0",
    );
    check(
        "let s=Symbol(),source={get a(){delete this[s];return 1;},[s]:7};let a={...source};a.a===1 && !Object.hasOwn(a,s)",
    );
}

#[test]
fn ordered_spreads_and_definitions_replace_values_and_bypass_setters() {
    check(
        "let log='',source={get a(){log+='a';return 2;}};function value(){log+='v';return 1;}let a={a:value(),...source,a:value(),...{b:3}};a.a===1 && a.b===3 && log==='vav' && Object.keys(a).join(',')==='a,b'",
    );
    check(
        "let setters=0;Object.defineProperty(Object.prototype,'x',{set(v){setters++;},configurable:true});let source={x:7};let a={set x(v){setters++;},...source};a.x===7 && setters===0 && Object.getOwnPropertyDescriptor(a,'x').writable",
    );
    check("let source={x:()=>{}},a={...source};a.x===source.x && a.x.name==='x'");
}

#[test]
fn proto_spread_keys_are_data_and_do_not_count_as_prototype_initializers() {
    check(
        "let proto={},source={['__proto__']:proto};let a={...source};Object.getPrototypeOf(a)===Object.prototype && Object.hasOwn(a,'__proto__') && a.__proto__===proto && Object.getOwnPropertyDescriptor(a,'__proto__').writable",
    );
    check(
        "let proto={x:7},source={['__proto__']:8};let a={__proto__:proto,...source};Object.getPrototypeOf(a)===proto && a.x===7 && a.__proto__===8",
    );
    check(
        "let a={__proto__:null,...{a:1},...{['__proto__']:2}};Object.getPrototypeOf(a)===null && a.a===1 && a.__proto__===2",
    );
}

#[test]
fn abrupt_getters_skip_later_sources_and_expressions() {
    check(
        "let log='',later=0,marker={},source={get a(){log+='a';return 1;},get b(){log+='b';throw marker;}};let caught=false;try{({...source,c:++later,...{get d(){later++;return 4;}}});}catch(e){caught=e===marker;}caught && log==='ab' && later===0",
    );
    check(
        "let called=0,caught=false;try{({...(()=>{called++;throw 7;})(),a:++called});}catch(e){caught=e===7;}caught && called===1",
    );
}

#[test]
fn copy_work_and_output_quotas_are_opt_in_host_failures() {
    let mut realm = Realm::new(Limits {
        max_properties: Some(100),
        ..Limits::default()
    });
    realm
        .eval("let flag=0,a={},b={};for(let i=0;i<60;i++){a['a'+i]=i;b['b'+i]=i;}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{({...a,...b});}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0,source={};").unwrap();
    // Build the source across evaluations so this budget tests CopyDataProperties.
    for i in 0..100 {
        realm.eval(&format!("source[{i}]={i}")).unwrap();
    }
    assert!(matches!(
        realm.eval("try{({...source});}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{({...Array});}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
