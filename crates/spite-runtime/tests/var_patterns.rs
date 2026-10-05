//! Var patterns use ordered ResolveBinding/PutValue, including with (14.3.3).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn vars_hoist_all_names_allow_duplicates_and_keep_parameter_aliasing() {
    check(
        "let before=x;var {a:x,b:[y=x],...rest}={a:2,b:[]};before===undefined && x===2 && y===2 && Object.keys(rest).length===0",
    );
    check("if(false)var {a:x,...rest}=missing;typeof x==='undefined' && typeof rest==='undefined'");
    check("var [x,x]=[1,2];var {a:x,b:x}={a:3,b:4};x===4");
    check("var {x,a:x,...x}={x:1,a:2,z:3};x.z===3 && !('a' in x) && !('x' in x)");
    check("function f(a){var [a]=[2];return arguments[0];}f(1)===2");
    check("function f(a=1){var [a]=[2];return a;}f()===2");
    check("function f(){return x;var {x}={x:1};}f()===undefined");
    check("var {a=b,b=2}={};a===undefined && b===2");
    check("var {fn=function(){},arrow=()=>1}={};fn.name==='fn' && arrow.name==='arrow'");
}

#[test]
fn object_single_names_resolve_after_key_conversion_and_before_getv() {
    check(
        "var x=1;let object={x:0},source={get a(){delete object.x;return 7;}};with(object)var {a:x}=source;object.x===7 && x===1",
    );
    check(
        "var x=1;let log=[],written,key={[Symbol.toPrimitive](){log.push('key');return 'a';}},source={get a(){log.push('get');return 7;}},object={set x(value){log.push('set');written=value;},[Symbol.unscopables]:{get x(){log.push('resolve');return false;}}};function rhs(){log.push('rhs');return source;}with(object)var {[key]:x}=rhs();written===7 && x===1 && log.join(',')==='rhs,key,resolve,get,set'",
    );
    check(
        "var x=1;let object={x:0};with(object)var {a:x=(delete object.x,7)}={};object.x===7 && x===1",
    );
    check(
        "var x=1;let object={x:0},source={get a(){delete object.x;return {b:7};}};with(object)var {a:{b:x}}=source;x===7 && !('x' in object)",
    );
}

#[test]
fn pattern_rhs_evaluation_precedes_binding_lookup_while_identifier_rhs_follows_it() {
    check(
        "var x=1;let object={x:0};function rhs(){delete object.x;return{x:7};}with(object)var {x}=rhs();x===7 && !('x' in object)",
    );
    check(
        "var x=1;let object={x:0};function rhs(){delete object.x;return 7;}with(object)var x=rhs();x===1 && object.x===7",
    );
    check(
        "var x=1;let object={x:0,[Symbol.unscopables]:{x:true}};with(object)var {a:x}={a:7};x===7 && object.x===0",
    );
}

#[test]
fn array_single_names_resolve_before_steps_and_keep_the_original_binding_object() {
    check(
        "var x=1;let log=[],object={x:0,[Symbol.unscopables]:{get x(){log.push('resolve');return false;}}},iterator={get next(){log.push('next');return function(){log.push('step');delete object.x;return{done:false,value:7};};},return(){log.push('return');return{};},[Symbol.iterator](){log.push('iterator');return this;}};with(object)var [x]=iterator;object.x===7 && x===1 && log.join(',')==='iterator,next,resolve,step,return'",
    );
    check(
        "var x=1;let object={x:0},iterator={next(){return{done:false,value:undefined};},return(){return{};},[Symbol.iterator](){return this;}};with(object)var [x=(delete object.x,7)]=iterator;object.x===7 && x===1",
    );
    check(
        "var x=1;let log=[],object={x:0,[Symbol.unscopables]:{get x(){log.push('resolve');return false;}}},iterator={next(){log.push('step');return{done:true};},[Symbol.iterator](){return this;}};with(object)var [x,x]=iterator;object.x===undefined && x===1 && log.join(',')==='resolve,step,resolve'",
    );
}

#[test]
fn rest_identifiers_resolve_before_copying_or_consuming_and_nested_rest_recurses() {
    check(
        "var rest=1;let object={rest:0},source={get a(){delete object.rest;return 7;}};with(object)var {...rest}=source;rest===1 && object.rest.a===7",
    );
    check(
        "var rest=1;let object={rest:0},n=0,iterator={next(){n++;delete object.rest;return n===1?{done:false,value:7}:{done:true};},[Symbol.iterator](){return this;}};with(object)var [...rest]=iterator;rest===1 && Array.isArray(object.rest) && object.rest[0]===7",
    );
    check("var [x,...{0:y,length}]=[1,2,3];x===1 && y===2 && length===2");
    check(
        "let s=Symbol('x'),other=Symbol('x');var {[s]:x,...rest}={a:1,[s]:2,[other]:3};x===2 && !(s in rest) && rest[other]===3 && rest.a===1",
    );
}

#[test]
fn loops_reuse_var_environments_and_assignment_failures_close_active_iterators() {
    check(
        "let fs=[];for(var [i]=[0],{j}={j:10};i<3;i++,j++)fs.push(()=>i+j);i===3 && j===13 && fs[0]()===16 && fs[2]()===16",
    );
    check(
        "let fs=[];for(var {x} of [{x:1},{x:2}])fs.push(()=>x);x===2 && fs[0]()===2 && fs[1]()===2",
    );
    check("for(var [first,...rest] in {ab:1,cd:2}) ;first==='c' && rest.join('')==='d'");
    check(
        "'use strict';let error,closed=0,iterator={next(){return{done:false,value:7};},return(){closed++;throw 8;},[Symbol.iterator](){return this;}};try{var [undefined]=iterator;}catch(e){error=e instanceof TypeError;}error && closed===1 && undefined===void 0",
    );
    check(
        "'use strict';let error,closed=0,iterator={next(){return{done:false,value:{a:7}};},return(){closed++;throw 8;},[Symbol.iterator](){return this;}};try{for(var {a:undefined} of iterator) ;}catch(e){error=e instanceof TypeError;}error && closed===1",
    );
    check("var x=9;let object={x:0};with(object)for(var [x] of [[1],[2]]) ;object.x===2 && x===9");
}

#[test]
fn elisions_step_failures_and_partial_initialization_follow_shared_iterator_rules() {
    check(
        "let steps=0,values=0,closed=0,iterator={next(){steps++;return{done:false,get value(){values++;return 7;}};},return(){closed++;return{};},[Symbol.iterator](){return this;}};var [,x,,]=iterator;x===7 && steps===3 && values===1 && closed===1",
    );
    check(
        "let error={},closed=0,result,iterator={next(){return{done:false,get value(){throw error;}};},return(){closed++;return{};},[Symbol.iterator](){return this;}};try{var [x]=iterator;}catch(e){result=e===error;}result && closed===0 && x===undefined",
    );
    check(
        "let error={},closed=0,result,iterator={next(){return{done:false,value:undefined};},return(){closed++;throw 8;},[Symbol.iterator](){return this;}};function fail(){throw error;}try{var [x=fail()]=iterator;}catch(e){result=e===error;}result && closed===1 && x===undefined",
    );
    check(
        "let result;try{var {a,b=missing,c}={a:1};}catch(e){result=e instanceof ReferenceError;}result && a===1 && b===undefined && c===undefined",
    );
}

#[test]
fn global_conflicts_precede_execution_and_captured_var_bindings_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let x=1,flag=0;").unwrap();
    assert!(matches!(
        realm.eval("flag=9;var {x}={x:2};"),
        Err(Error::Exception {
            kind: ExceptionKind::SyntaxError,
            ..
        })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    realm.eval("let fs=[];function f(){var {value,fn=()=>value}={value:{n:7}};return fn;}fs.push(f());").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("fs[0]().n"), Ok(Value::Number(7.0)));
    let names = (0..12000)
        .map(|i| format!("p{i}"))
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        realm.eval(&format!("var {{{names}}}={{}};p11999===undefined")),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn opt_in_abort_skips_language_cleanup_and_restores_with_and_loop_scopes() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0,closed=0,inner={next(){return{done:false,value:1};},return(){closed++;return{};},[Symbol.iterator](){return this;}},outer={next(){return{done:false,value:inner};},return(){closed++;return{};},[Symbol.iterator](){return this;}};").unwrap();
    assert!(matches!(
        realm.eval(
            "try{with({flag:7})for(var [...rest] of outer)flag=3;}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && closed===0 && rest===undefined"),
        Ok(Value::Boolean(true))
    );
}
