//! AggregateError's ordered initialization and synchronous IteratorToList (20.5.7).

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn prototype_message_cause_and_iterator_are_observed_in_order() {
    check(
        "let log='',proto={},N=function(){}.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';return proto;}});let message={toString(){log+='m';return 'detail';}},options={get cause(){log+='c';return 7;}},errors={get [Symbol.iterator](){log+='i';return function(){log+='j';return {get next(){log+='n';return function(){log+='s';return {get done(){log+='d';return true;},get value(){throw 8;}};};}};};}};let e=Reflect.construct(AggregateError,[errors,message,options],N);log==='pmcijnsd' && Object.getPrototypeOf(e)===proto && Error.isError(e) && e.message==='detail' && e.cause===7 && e.errors.length===0",
    );
    check(
        "let log='',errors={get [Symbol.iterator](){log+='i';throw 3;}},message={toString(){log+='m';throw 1;}},options={get cause(){log+='c';throw 2;}},caught=false;try{AggregateError(errors,message,options);}catch(e){caught=e===1;}caught && log==='m'",
    );
    check(
        "let log='',errors={get [Symbol.iterator](){log+='i';throw 3;}},options={get cause(){log+='c';throw 2;}},caught=false;try{AggregateError(errors,'detail',options);}catch(e){caught=e===2;}caught && log==='c'",
    );
    check(
        "let log='',caught=false;try{AggregateError(undefined,{toString(){log+='m';throw 1;}});}catch(e){caught=e===1;}caught && log==='m'",
    );
    check(
        "let marker={},caught=false;try{AggregateError({[Symbol.iterator](){throw marker;}});}catch(e){caught=e===marker;}caught",
    );
}

#[test]
fn iteration_caches_next_and_never_closes_on_exhaustion_or_step_failure() {
    check(
        "let log='',i=0,it={get next(){log+='n';return function(){log+='s';if(this!==it)throw 9;return {get done(){log+='d';return i===2;},get value(){log+='v';return i++;}};};},get return(){throw 8;}},errors={[Symbol.iterator](){if(this!==errors)throw 7;return it;}};let a=AggregateError(errors).errors;log==='nsdvsdvsd' && a.length===2 && a[0]===0 && a[1]===1",
    );
    for step in [
        "throw marker;",
        "return {get done(){throw marker;}};",
        "return {done:false,get value(){throw marker;}};",
    ] {
        check(&format!(
            "let marker={{}},closed=false,caught=false,it={{next(){{{step}}},get return(){{closed=true;throw 8;}}}};try{{AggregateError({{[Symbol.iterator](){{return it;}}}});}}catch(e){{caught=e===marker;}}caught && !closed"
        ));
    }
    check(
        "let closed=false,caught=false;try{AggregateError({[Symbol.iterator](){return {next(){return 1;},return(){closed=true;}};}});}catch(e){caught=e instanceof TypeError;}caught && !closed",
    );
    check(
        "let closed=false,caught=false;try{AggregateError({[Symbol.iterator](){return {get next(){throw 7;},get return(){closed=true;}};}});}catch(e){caught=e===7;}caught && !closed",
    );
}

#[test]
fn errors_are_an_intrinsic_dense_copy_without_value_conversion_or_species() {
    check(
        "let marker={toString(){throw 1;},valueOf(){throw 2;}},input=[marker,,undefined];Object.defineProperty(input,'constructor',{get(){throw 3;}});let e=AggregateError(input),f=AggregateError(input);input[0]=7;e.errors!==input && e.errors!==f.errors && e.errors.length===3 && e.errors[0]===marker && e.errors[1]===undefined && e.errors[2]===undefined && Object.keys(e.errors).join(',')==='0,1,2' && Array.isArray(e.errors)",
    );
    check(
        "let proto=Array.prototype,Original=Array;Object.defineProperty(proto,'0',{set(){throw 7;},configurable:true});Array=function(){throw 8;};let e=AggregateError({[Symbol.iterator](){let n=0;return {next(){return n++===0?{value:4,done:false}:{done:true};}};}});Object.getPrototypeOf(e.errors)===proto && e.errors[0]===4 && e.errors.length===1 && Original.isArray(e.errors)",
    );
    check(
        "let e=AggregateError('a😀\\uD800');e.errors.length===3 && e.errors[0]==='a' && e.errors[1]==='😀' && e.errors[2]==='\\uD800'",
    );
    for input in [
        "undefined",
        "null",
        "true",
        "1",
        "1n",
        "Symbol()",
        "{}",
        "{next(){return {done:true};}}",
        "{[Symbol.iterator]:1}",
        "{[Symbol.iterator](){return 1;}}",
    ] {
        assert!(
            matches!(
                Realm::default().eval(&format!("AggregateError({input})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{input}"
        );
    }
}

#[test]
fn calls_construction_causes_and_descriptors_follow_the_error_family() {
    check(
        "let a=AggregateError([1],'detail',{cause:7}),b=new AggregateError([2],'detail');a!==b && a instanceof AggregateError && a instanceof Error && Error.isError(a) && a.toString()==='AggregateError: detail' && a.cause===7 && b.errors[0]===2 && AggregateError([]).errors.length===0",
    );
}

#[test]
fn constructor_and_instance_descriptors_are_standard() {
    check(
        "let F=AggregateError.bind({},[7],'bound'),a=new F,b=AggregateError.call(null,[8],'call');a instanceof AggregateError && a.message==='bound' && a.errors[0]===7 && b.errors[0]===8 && b.message==='call'",
    );
    check(
        "Object.getPrototypeOf(AggregateError)===Error && Object.getPrototypeOf(AggregateError.prototype)===Error.prototype && AggregateError.prototype.constructor===AggregateError && AggregateError.prototype.name==='AggregateError' && AggregateError.prototype.message==='' && !('errors' in AggregateError.prototype) && !Error.isError(AggregateError.prototype) && ({}).toString.call(AggregateError.prototype)==='[object Object]' && ({}).toString.call(AggregateError([]))==='[object Error]'",
    );
    check(
        "let e=AggregateError([],undefined,{__proto__:{cause:undefined}});!Object.hasOwn(e,'message') && Object.hasOwn(e,'cause') && e.cause===undefined && Reflect.ownKeys(e).join(',')==='cause,errors'",
    );
    check(
        "let e=AggregateError([],null,{cause:3});e.message==='null' && Reflect.ownKeys(e).join(',')==='message,cause,errors' && delete e.errors && !('errors' in e)",
    );
    for options in ["undefined", "null", "true", "1", "1n", "'text'", "{}"] {
        check(&format!(
            "!('cause' in AggregateError([],undefined,{options}))"
        ));
    }
    let mut realm = Realm::default();
    for (source, fields) in [
        (
            "AggregateError",
            vec![
                ("name", Value::String("AggregateError".into()), false, true),
                ("length", Value::Number(2.0), false, true),
            ],
        ),
        (
            "AggregateError.prototype",
            vec![
                ("name", Value::String("AggregateError".into()), true, true),
                ("message", Value::String("".into()), true, true),
            ],
        ),
        (
            "AggregateError([], 'detail', {cause:7})",
            vec![
                ("message", Value::String("detail".into()), true, true),
                ("cause", Value::Number(7.0), true, true),
            ],
        ),
    ] {
        let Value::Object(object) = realm.eval(source).unwrap() else {
            panic!("object")
        };
        for (name, value, writable, configurable) in fields {
            let desc = realm
                .inspect_object(&object)
                .unwrap()
                .own_property(&JsString::from(name))
                .unwrap()
                .as_data()
                .unwrap();
            assert_eq!(desc.value, value);
            assert_eq!(desc.writable, writable);
            assert_eq!(desc.configurable, configurable);
            assert!(!desc.enumerable);
        }
    }
    check(
        "let d=Object.getOwnPropertyDescriptor(AggregateError,'prototype');!d.writable && !d.enumerable && !d.configurable",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(AggregateError([]),'errors');d.writable && !d.enumerable && d.configurable && Array.isArray(d.value) && d.value.length===0",
    );
}

#[test]
fn retained_intrinsics_and_list_values_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let Saved=AggregateError,e=Saved([{message:'inner'}], 'outer', {cause:{message:'cause'}});delete globalThis.AggregateError;delete globalThis.Error;delete globalThis.Array;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("e.errors[0].message==='inner' && e.cause.message==='cause' && Saved.isError(e) && Saved([3]).errors[0]===3"), Ok(Value::Boolean(true)));
}

#[test]
fn default_list_size_is_unrestricted_and_opted_in_host_failures_bypass_cleanup() {
    check(
        "let n=0,e=AggregateError({[Symbol.iterator](){return {next(){return n<5000?{value:n++,done:false}:{done:true};}};}});e.errors.length===5000 && e.errors[4999]===4999",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0,closed=false;").unwrap();
    assert!(matches!(realm.eval("try{AggregateError({[Symbol.iterator](){return {next(){return {done:false,value:1};},return(){closed=true;}};}});}catch{flag=1;}finally{flag=2;}"), Err(Error::Limit {..})));
    assert_eq!(realm.eval("flag===0 && !closed"), Ok(Value::Boolean(true)));
    let mut realm = Realm::default();
    realm.eval("let flag=0,closed=false;").unwrap();
    assert!(matches!(realm.eval("try{AggregateError({[Symbol.iterator](){return {next(){Function('function* gap(){}');},return(){closed=true;}};}});}catch{flag=1;}finally{flag=2;}"), Err(Error::Unsupported {..})));
    assert_eq!(realm.eval("flag===0 && !closed"), Ok(Value::Boolean(true)));
}
