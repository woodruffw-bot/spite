//! Ordered destructuring writes, edition-17 references, and iterator completion (13.15.5).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn assignments_return_the_original_rhs_and_write_nested_repeated_and_rest_targets() {
    check(
        "let x,y,xs,rest,source={a:[1,2,3],z:4};let result=({a:[x,y,...xs],...rest}=source);result===source && x===1 && y===2 && xs[0]===3 && rest.z===4",
    );
    check("let x;[x,x]=[1,2];({a:x,b:x}={a:3,b:4});x===4");
    check(
        "var eval,arguments;[eval=3,arguments=4]=[];({eval,arguments}={eval:5,arguments:6});eval===5 && arguments===6",
    );
    check("let x=1,y=2;[x,y]=[y,x];x===2 && y===1");
    check("let a,b,source=[2];let result=([a]=[b]=source);result===source && a===2 && b===2");
    check("let object={};({x:object.x=2,...object.rest}={y:3});object.x===2 && object.rest.y===3");
    check("let x,y;({__proto__:x,__proto__:y}={__proto__:null});x===undefined && y===undefined");
    check("let x,n;[x,...{length:n}]=[1,2,3];x===1 && n===2");
    check("let key,value;for([key,value] of [[1,2],[3,4]]) ;key===3 && value===4");
    check(
        "let first,tail,log=[];for([first,...tail] in {ab:1,cd:2})log.push(first+tail[0]);log.join(',')==='ab,cd'",
    );
    check(
        "var x=1;let object={x:0},source={get a(){delete object.x;return 7;}};with(object)({a:x}=source);object.x===7 && x===1",
    );
}

#[test]
fn reference_evaluation_precedes_source_reads_but_edition17_key_conversion_waits_for_put() {
    check(
        "let log=[],object={},key={[Symbol.toPrimitive](){log.push('convert');return 'x';}},source={get a(){log.push('get');return undefined;}};function target(){log.push('target');return object;}function rhs(){log.push('rhs');return source;}({a:target()[key]=(log.push('default'),7)}=rhs());object.x===7 && log.join(',')==='rhs,target,get,default,convert'",
    );
    check(
        "let log=[],object={},targetKey={[Symbol.toPrimitive](){log.push('target-key');return 'x';}},sourceKey={[Symbol.toPrimitive](){log.push('source-key');return 'a';}},source={get a(){log.push('get');return 7;}};({[sourceKey]:object[targetKey]}=source);object.x===7 && log.join(',')==='source-key,get,target-key'",
    );
    check(
        "let key='x',object={},targetKey={[Symbol.toPrimitive](){return key;}},source={get a(){key='y';return 7;}};({a:object[targetKey]}=source);object.y===7 && !('x' in object)",
    );
    check(
        "let log=[],source={get a(){log.push('get');return 7;}},result;try{({a:null.x}=source);}catch(e){result=e instanceof TypeError;}result && log.join(',')==='get'",
    );
    check(
        "let log=[],object={},source={get a(){log.push('get');return {b:7};}};function target(){log.push('target');return object;}({a:{b:target().x}}=source);object.x===7 && log.join(',')==='get,target'",
    );
}

#[test]
fn array_targets_evaluate_before_steps_even_after_exhaustion_and_keep_original_references() {
    check(
        "let log=[],object={},key={[Symbol.toPrimitive](){log.push('convert');return 'x';}},source={next(){log.push('step');return{done:false,value:undefined};},return(){log.push('close');return{};},[Symbol.iterator](){return this;}};function target(){log.push('target');return object;}[target()[key]=(log.push('default'),7)]=source;object.x===7 && log.join(',')==='target,step,default,convert,close'",
    );
    check(
        "let log=[],object={},source={next(){log.push('step');return{done:true};},[Symbol.iterator](){return this;}};function target(){log.push('target');return object;}[target().x,target().y]=source;object.x===undefined && object.y===undefined && log.join(',')==='target,step,target'",
    );
    check(
        "let first={},second={},object=first,source={next(){object=second;return{done:false,value:7};},return(){return{};},[Symbol.iterator](){return this;}};[object.x]=source;first.x===7 && !('x' in second)",
    );
    check(
        "let log=[],object={},source={next(){log.push('step');return{done:true};},[Symbol.iterator](){return this;}};function target(){log.push('target');return object;}[...target().xs]=source;object.xs.length===0 && log.join(',')==='target,step'",
    );
}

#[test]
fn object_rest_resolves_before_live_copy_and_uses_exact_key_exclusion_and_own_data() {
    check(
        "let log=[],object={},source={get a(){log.push('get');return 7;}};function target(){log.push('target');return object;}({...target().rest}=source);object.rest.a===7 && log.join(',')==='target,get'",
    );
    check(
        "let a,rest,s=Symbol('s'),source={a:1,b:2,[s]:3};Object.defineProperty(source,'hidden',{value:4});({a,...rest}=source);a===1 && rest.b===2 && rest[s]===3 && !('a' in rest) && !('hidden' in rest) && Object.getPrototypeOf(rest)===Object.prototype",
    );
    check(
        "let seen=[],rest,source={get a(){seen.push('a');delete this.b;this.c=3;return 1;},b:2};({...rest}=source);seen.join(',')==='a' && rest.a===1 && !('b' in rest) && !('c' in rest)",
    );
    check(
        "let object={set x(value){throw 7;}};let result;try{({...object.x}={a:1});}catch(e){result=e===7;}result",
    );
}

#[test]
fn default_name_inference_distinguishes_identifier_parentheses_and_property_targets() {
    check(
        "let a,b,object={};[a=function(){},(b)=function(){},object.x=function(){}]=[];a.name==='a' && b.name==='' && object.x.name===''",
    );
    check(
        "let a,b,object={};({a=()=>0,b:(b)=()=>0,c:object.x=()=>0}={});a.name==='a' && b.name==='' && object.x.name===''",
    );
    check("let x=0,y=0;[x=1,y=x+1]=[];x===1 && y===2");
    check("let x=0,n=0;({x=(n++,7)}={x:null});x===null && n===0");
    check("let x;[...{length:x}]=[1,2,3];x===3");
}

#[test]
fn iterator_close_orders_nested_cleanup_and_preserves_incoming_write_and_default_errors() {
    check(
        "let log=[],error={},object={set x(value){throw error;}},source={next(){return{done:false,value:7};},return(){log.push('close');throw 8;},[Symbol.iterator](){return this;}};let result;try{[object.x]=source;}catch(e){result=e===error;}result && log.join(',')==='close'",
    );
    check(
        "let log=[],source={next(){return{done:false,value:7};},return(){log.push('close');return{};},[Symbol.iterator](){return this;}};function target(){throw 7;}let result;try{[target().x]=source;}catch(e){result=e===7;}result && log.join(',')==='close'",
    );
    check(
        "let log=[],error={},inner={next(){return{done:false,value:undefined};},return(){log.push('inner');throw 8;},[Symbol.iterator](){return this;}},outer={next(){return{done:false,value:inner};},return(){log.push('outer');throw 9;},[Symbol.iterator](){return this;}};function fail(){throw error;}let x,result;try{[[x=fail()]]=outer;}catch(e){result=e===error;}result && log.join(',')==='inner,outer'",
    );
    check(
        "let closed=0,source={next(){throw 7;},return(){closed++;return{};},[Symbol.iterator](){return this;}};let x,result;try{[x]=source;}catch(e){result=e===7;}result && closed===0",
    );
    check(
        "let calls=0,source={next(){return{done:false,get value(){throw 7;}};},return(){calls++;return{};},[Symbol.iterator](){return this;}};let x,result;try{[x]=source;}catch(e){result=e===7;}result && calls===0",
    );
}

#[test]
fn assignments_preserve_strict_writes_partial_effects_and_outer_loop_closing() {
    check(
        "let x=0,object={};Object.defineProperty(object,'y',{value:0});let result;try{(function(){'use strict';[x,object.y]=[1,2];})();}catch(e){result=e instanceof TypeError;}result && x===1 && object.y===0",
    );
    check(
        "let x=0,object={set y(value){throw 7;}},closed=0,source={next(){return{done:false,value:[1,2]};},return(){closed++;return{};},[Symbol.iterator](){return this;}};let result;try{for([x,object.y] of source) ;}catch(e){result=e===7;}result && closed===1 && x===1",
    );
    check("let x=0;[x,created]=[1,2];x===1 && created===2");
    let mut realm = Realm::default();
    assert!(matches!(
        realm.eval("'use strict';let x=0;[x,missing]=[1,2];"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    assert_eq!(realm.eval("other=3"), Ok(Value::Number(3.0)));
}

#[test]
fn partial_assignment_closures_survive_collection_and_reentrant_defaults_share_stack_guards() {
    let mut realm = Realm::default();
    realm
        .eval("let x,saved,y;try{[x={n:1},saved=()=>x,y=missing]=[];}catch{}")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("saved().n"), Ok(Value::Number(1.0)));
    assert_eq!(realm.eval("y"), Ok(Value::Undefined));
    realm
        .eval("let flag=0;function f(){let x;[x=f()]=[];}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{f();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("after=3"), Ok(Value::Number(3.0)));
}

#[test]
fn patterns_have_no_default_flat_quota_and_opted_in_aborts_skip_iterator_cleanup() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0,closed=0,x,source={next(){return{done:false,value:1};},return(){closed++;return{};},[Symbol.iterator](){return this;}};").unwrap();
    assert!(matches!(
        realm.eval("try{[...x]=source;}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag+closed"), Ok(Value::Number(0.0)));
    let names = (0..12000)
        .map(|i| format!("p{i}"))
        .collect::<Vec<_>>()
        .join(",");
    check(&format!(
        "let {names};({{{names}}}={{}});p11999===undefined"
    ));
}
