//! Catch destructuring uses BindingInitialization and restores scopes (14.15.2).

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn object_patterns_read_own_inherited_and_nonenumerable_properties_in_order() {
    check(
        "let source=Object.create({a:1});Object.defineProperty(source,'b',{value:2});let result;try{throw source;}catch({a:x,b:y,c:z=3}){result=x+y+z;}result===6",
    );
    check(
        "let log=[],key={[Symbol.toPrimitive](hint){log.push(hint);return 'a';}},source={get a(){log.push('get');return undefined;},get b(){log.push('b');return 7;}},result;function fallback(){log.push('default');return 3;}try{throw source;}catch({[key]:x=fallback(),b:y}){result=x+y;}result===10 && log.join(',')==='string,get,default,b'",
    );
    check(
        "let called=0,result;function fallback(){called++;return 7;}try{throw{a:null,b:false,c:0};}catch({a=fallback(),b=fallback(),c=fallback(),d=fallback()}){result=a===null && b===false && c===0 && d===7;}result && called===1",
    );
    check(
        "let source={get a(){delete source.b;source.c=3;return 1;},b:2},result;try{throw source;}catch({a,b=7,c}){result=a===1 && b===7 && c===3;}result",
    );
}

#[test]
fn defaults_have_parameter_scope_tdz_and_named_function_initialization() {
    check(
        "let result;try{throw{};}catch({a=2,b=a+3,f=function(){return b;},g=()=>a}){result=b===5 && f()===5 && f.name==='f' && g.name==='g' && g()===2;}result",
    );
    check(
        "let b=9,flag=0,result;try{try{throw{};}catch({a=b,b=2}){flag=1;}}catch(e){result=e instanceof ReferenceError;}result && b===9 && flag===0",
    );
    check(
        "let a='a',result;try{try{throw{};}catch({[a]:a}){}}catch(e){result=e instanceof ReferenceError;}result && a==='a'",
    );
    check(
        "let f,result;try{throw{a:[2],b:{}};}catch({a:[x],b:{y=x},c:fallback=(function(){return y;})}){f=fallback;result=x===2 && y===2 && fallback.name==='fallback';}result && f()===2",
    );
    check(
        "let result;try{throw[];}catch([f=function(){},g=()=>1]){result=f.name==='f' && g.name==='g';}result",
    );
}

#[test]
fn object_rest_excludes_exact_keys_and_copies_live_own_enumerable_data() {
    check(
        "let a=Symbol('x'),b=Symbol('x'),source={x:1,[a]:2,[b]:3},rest,result;try{throw source;}catch({[a]:v,...r}){rest=r;result=v===2;}result && rest.x===1 && !(a in rest) && rest[b]===3 && Object.getPrototypeOf(rest)===Object.prototype",
    );
    check(
        "let source=Object.create({inherited:9});source.a=1;Object.defineProperty(source,'hidden',{value:2});Object.defineProperty(source,'__proto__',{value:7,enumerable:true});let rest;try{throw source;}catch({a,...r}){rest=r;}!('a' in rest) && !('hidden' in rest) && !Object.hasOwn(rest,'inherited') && Object.hasOwn(rest,'__proto__') && rest.__proto__===7 && Object.getPrototypeOf(rest)===Object.prototype",
    );
    check(
        "let calls=0,source={get a(){calls++;return 1;},get b(){delete source.c;source.added=4;return 2;},c:3},rest;try{throw source;}catch({a,...r}){rest=r;}calls===1 && rest.b===2 && !('c' in rest) && !('added' in rest) && Object.getOwnPropertyDescriptor(rest,'b').writable",
    );
    check(
        "let source={get a(){source.b=2;return 1;}},rest;try{throw source;}catch({a,...r}){rest=r;}rest.b===2",
    );
    check(
        "let key={[Symbol.toPrimitive](){return 'x';}},rest,result;try{throw{x:1,y:2};}catch({[key]:a,x:b,...r}){rest=r;result=a===1 && b===1;}result && !('x' in rest) && rest.y===2",
    );
}

#[test]
fn object_patterns_preserve_primitive_receivers_and_nullish_errors() {
    check(
        "let result;Object.defineProperty(Number.prototype,'x',{get:function(){'use strict';return this;}});try{throw 7;}catch({x}){result=x;}result===7",
    );
    check(
        "let result;try{throw 'abc';}catch({0:first,length,...rest}){result=first==='a' && length===3 && rest[1]==='b' && rest[2]==='c' && !('0' in rest) && !Object.hasOwn(rest,'length');}result",
    );
    check(
        "let count=0;for(let value of [true,7,7n,Symbol()]){try{throw value;}catch({}){count++;}}count===4",
    );
    check(
        "let flag=0;for(let value of [null,undefined]){try{try{throw value;}catch({}){flag=9;}finally{flag++;}}catch(e){if(e instanceof TypeError)flag++;}}flag===4",
    );
    check(
        "let calls=0,source={[Symbol.toPrimitive](){calls++;throw 7;}},result;try{throw source;}catch({}){result=true;}result && calls===0",
    );
}

#[test]
fn array_patterns_consume_iterators_and_elisions_skip_value_getters() {
    check(
        "let result;try{throw [1,,3,4];}catch([a,b=2,...rest]){result=a===1 && b===2 && rest.join(',')==='3,4';}result",
    );
    check(
        "let result;try{throw '😀x';}catch([first,...rest]){result=first==='😀' && rest.join('')==='x';}result",
    );
    check(
        "let result;try{throw [1,2,3];}catch([x,...[y,z]]){result=x===1 && y===2 && z===3;}result",
    );
    check(
        "let result;try{throw [1,2,3];}catch([x,...{length,0:y}]){result=x===1 && length===2 && y===2;}result",
    );
    check(
        "let steps=0,values=0,closed=0,iterator={next(){steps++;return {done:false,get value(){values++;return 7;}};},return(){closed++;return {};},[Symbol.iterator](){return this;}},result;try{throw iterator;}catch([,x,,]){result=x===7;}result && steps===3 && values===1 && closed===1",
    );
    check(
        "let calls=0,iterator={next(){calls++;return{done:true,get value(){throw 7;}};},return(){throw 8;},[Symbol.iterator](){return this;}},result;try{throw iterator;}catch([,x=1,,y=x]){result=x===1 && y===1;}result && calls===1",
    );
}

#[test]
fn arrays_cache_next_and_close_on_partial_normal_consumption() {
    check(
        "let gets=0,calls=0,closed=0,iterator={get next(){gets++;return function(){calls++;return{done:false,value:calls};};},return(){closed++;return {};},[Symbol.iterator](){return this;}},result;try{throw iterator;}catch([x,y]){result=x===1 && y===2;}result && gets===1 && calls===2 && closed===1",
    );
    check(
        "let next=0,closed=0,iterator={next(){next++;throw 7;},return(){closed++;return {};},[Symbol.iterator](){return this;}};try{throw iterator;}catch([]){}next===0 && closed===1",
    );
    check(
        "let closed=0,result,iterator={next(){return{value:7,done:false};},return(){closed++;throw 8;},[Symbol.iterator](){return this;}};try{try{throw iterator;}catch([x]){}}catch(e){result=e===8;}result && closed===1",
    );
    check(
        "let result,iterator={next(){return{done:false,value:7};},return(){return 1;},[Symbol.iterator](){return this;}};try{try{throw iterator;}catch([x]){}}catch(e){result=e instanceof TypeError;}result",
    );
}

#[test]
fn iterator_step_errors_skip_closing_and_nested_binding_throws_preserve_identity() {
    for next in [
        "throw error;",
        "return 7;",
        "return{get done(){throw error;}};",
        "return{done:false,get value(){throw error;}};",
    ] {
        check(&format!(
            "let error={{}},closed=0,result,iterator={{next(){{{next}}},return(){{closed++;return {{}};}},[Symbol.iterator](){{return this;}}}};try{{try{{throw iterator;}}catch([x]){{}}}}catch(e){{result=e===error || e instanceof TypeError;}}result && closed===0"
        ));
    }
    check(
        "let error={},closed=0,result,iterator={next(){return{done:false,value:undefined};},return(){closed++;throw 8;},[Symbol.iterator](){return this;}};function fail(){throw error;}try{try{throw iterator;}catch([x=fail()]){}}catch(e){result=e===error;}result && closed===1",
    );
    check(
        "let order=[],error={},inner={next(){return{done:false,value:undefined};},return(){order.push('inner');return {};},[Symbol.iterator](){return this;}},outer={next(){return{done:false,value:inner};},return(){order.push('outer');return {};},[Symbol.iterator](){return this;}};function fail(){throw error;}let result;try{try{throw outer;}catch([[x=fail()]]){}}catch(e){result=e===error;}result && order.join(',')==='inner,outer'",
    );
    check(
        "let error={},closed=0,result,iterator={next(){return{done:false,value:null};},return(){closed++;return {};},[Symbol.iterator](){return this;}};try{try{throw iterator;}catch([{}]){}}catch(e){result=e instanceof TypeError;}result && closed===1",
    );
}

#[test]
fn scopes_restore_on_binding_failure_finalizers_and_captured_closures_survive_gc() {
    check(
        "let x=7,flag=0,result;try{try{throw{};}catch({x=y,y=1}){flag=9;}finally{flag=x;}}catch(e){result=e instanceof ReferenceError;}result && flag===7 && x===7 && typeof y==='undefined'",
    );
    check(
        "let result;try{+1n;}catch({name,message,constructor}){result=name==='TypeError' && typeof message==='string' && constructor===TypeError;}result",
    );
    let mut realm = Realm::default();
    realm
        .eval("let f;try{throw{a:{value:7}};}catch({a,...rest}){f=function(){return a.value;};}")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f()"), Ok(Value::Number(7.0)));
    realm
        .eval("let saved;try{try{throw{};}catch({a=(saved=()=>b,0),b=missing}){}}catch{};")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("let result;try{saved();}catch(e){result=e instanceof ReferenceError;}result"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn unlimited_flat_patterns_and_opted_in_host_aborts_keep_distinct_semantics() {
    let mut realm = Realm::default();
    let names = (0..12000)
        .map(|i| format!("p{i}"))
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        realm.eval(&format!(
            "try{{throw{{}};}}catch({{{names}}}){{p11999===undefined;}}"
        )),
        Ok(Value::Boolean(true))
    );
    let mut limited = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    limited.eval("let flag=0,closed=0,iterator={next(){return{done:false,value:1};},return(){closed++;return {};},[Symbol.iterator](){return this;}};").unwrap();
    assert!(matches!(
        limited.eval("try{try{throw iterator;}catch([...rest]){}}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        limited.eval("flag===0 && closed===0 && typeof rest==='undefined'"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::default();
    realm.eval("let flag=0,closed=0,iterator={next(){return{done:false,value:undefined};},return(){closed++;return {};},[Symbol.iterator](){return this;}};").unwrap();
    assert!(matches!(realm.eval("try{try{throw iterator;}catch([x=Function('function* gap(){}')]){}}catch{flag=1;}finally{flag=2;}"),Err(Error::Unsupported{..})));
    assert_eq!(
        realm.eval("flag===0 && closed===0 && typeof x==='undefined'"),
        Ok(Value::Boolean(true))
    );
}
