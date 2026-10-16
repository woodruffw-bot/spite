//! Lexical declaration patterns share BindingInitialization (14.3.1, 14.7).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn mixed_declarations_initialize_in_order_with_tdz_and_named_defaults() {
    check(
        "let a=2,{b=a+1,c=()=>b}={},[d=b+1,...rest]=[];a===2 && b===3 && c()===3 && c.name==='c' && d===4 && rest.length===0",
    );
    check(
        "const fn=function(){},{a=(function(){}),b=(0,function(){})}={};fn.name==='fn' && a.name==='a' && b.name!== 'b'",
    );
    check(
        "let calls=0;function fallback(){calls++;return 9;}let {a=fallback(),b=fallback(),c=fallback(),d=fallback()}={a:null,b:false,c:0};a===null && b===false && c===0 && d===9 && calls===1",
    );
    check(
        "let x=7,result;try{let {a=x,x=1}={};}catch(e){result=e instanceof ReferenceError;}result && x===7",
    );
    check(
        "let key='a',result;try{let {[key]:key}={a:1};}catch(e){result=e instanceof ReferenceError;}result && key==='a'",
    );
    check("let result=false;with({x:9}) {let {x}={x:2};result=x===2;}result");
}

#[test]
fn object_keys_and_rest_preserve_receivers_identity_and_live_descriptors() {
    check(
        "let log=[],key={[Symbol.toPrimitive](hint){log.push(hint);return 'a';}},source={get a(){log.push('get');return undefined;},get b(){log.push('b');return 7;}};function fallback(){log.push('default');return 3;}const {[key]:x=fallback(),b:y}=source;x+y===10 && log.join(',')==='string,get,default,b'",
    );
    check(
        "let s=Symbol('x'),other=Symbol('x'),source={x:1,[s]:2,[other]:3};const {[s]:v,...rest}=source;v===2 && !(s in rest) && rest[other]===3 && rest.x===1 && Object.getPrototypeOf(rest)===Object.prototype",
    );
    check(
        "let source={get a(){delete source.b;source.c=3;return 1;},b:2};const {a,...rest}=source;a===1 && !('b' in rest) && rest.c===3 && Object.getOwnPropertyDescriptor(rest,'c').writable",
    );
    check(
        "Object.defineProperty(Number.prototype,'x',{get:function(){'use strict';return this;}});let {x}=7;x===7",
    );
    check(
        "const {0:first,...rest}='abc';first==='a' && rest[1]==='b' && rest[2]==='c' && !('0' in rest)",
    );
}

#[test]
fn arrays_use_iterators_skip_elision_values_and_close_after_partial_consumption() {
    check(
        "let steps=0,values=0,closed=0,iterator={next(){steps++;return{done:false,get value(){values++;return 7;}};},return(){closed++;return{};},[Symbol.iterator](){return this;}};const [,x,,]=iterator;x===7 && steps===3 && values===1 && closed===1",
    );
    check(
        "let closed=0,iterator={next(){throw 7;},return(){closed++;return{};},[Symbol.iterator](){return this;}};let []=iterator;closed===1",
    );
    check("const [first,...rest]='😀x';first==='😀' && rest.join('')==='x'");
    check("let [a,b=2,...{0:c,length}]=[1,,3,4];a===1 && b===2 && c===3 && length===2");
    check(
        "let calls=0,iterator={next(){calls++;return{done:true,get value(){throw 7;}};},return(){throw 8;},[Symbol.iterator](){return this;}};const [,x=1,,y=x]=iterator;x===1 && y===1 && calls===1",
    );
}

#[test]
fn immutable_patterns_and_function_arguments_shadowing_follow_declarations() {
    for directive in ["", "'use strict';"] {
        check(&format!(
            "{directive}let {{x}}={{x:1}};x=2;const [y]=[3];let result;try{{y=4;}}catch(e){{result=e instanceof TypeError;}}result && x===2 && y===3"
        ));
    }
    check("function f(){let {arguments}={arguments:7};return arguments;}f(1,2)===7");
    check("function f(a=2){let {arguments}={arguments:7};return a+arguments;}f()===9");
    check("function f(){const {a=()=>arguments[0]}={};return a();}f(7)===7");
    check("let f=()=>{const {a=()=>7}={};return a;};f()()===7");
}

#[test]
fn three_clause_loops_copy_every_let_name_and_restore_scope_on_all_exits() {
    check(
        "let fs=[];for(let [i]=[0],{j}={j:10};i<3;i++,j++)fs.push(()=>i+j);fs[0]()===10 && fs[1]()===12 && fs[2]()===14 && typeof i==='undefined' && typeof j==='undefined'",
    );
    check(
        "let initial,body;for(let [i=0,f=()=>i]=[];i<1;i++){initial=f;body=()=>i;}initial()===0 && body()===0",
    );
    check(
        "let i=9,result;try{for(let [i]=[j],j=1;false;) ;}catch(e){result=e instanceof ReferenceError;}result && i===9 && typeof j==='undefined'",
    );
    check("let fs=[],n=0;for(const {x}={x:7};n<2;n++)fs.push(()=>x);fs[0]()===7 && fs[1]()===7");
    check("let calls=0;for(let {}={};calls<3;calls++){}calls===3");
    check(
        "let fs=[];for(let [i]=[0];i<3;i++){fs.push(()=>i);continue;}fs[0]()===0 && fs[1]()===1 && fs[2]()===2",
    );
}

#[test]
fn in_and_of_patterns_have_rhs_tdz_fresh_scopes_and_nested_iterator_closing() {
    check(
        "let fs=[];for(const {x} of [{x:1},{x:2}])fs.push(()=>x);fs[0]()===1 && fs[1]()===2 && typeof x==='undefined'",
    );
    check(
        "let fs=[];for(let [x,y=x] of [[1],[2]]){y++;fs.push(()=>x+y);}fs[0]()===3 && fs[1]()===5",
    );
    check(
        "let fs=[];for(const [first,...rest] in {ab:1,cd:2})fs.push(()=>first+rest.join(''));fs[0]()==='ab' && fs[1]()==='cd'",
    );
    check(
        "let x=[],result;try{for(let [x] of x) ;}catch(e){result=e instanceof ReferenceError;}result && x.length===0",
    );
    check(
        "let key={},result;try{for(const {key} in key) ;}catch(e){result=e instanceof ReferenceError;}result && typeof key==='object'",
    );
    check("let count=0;for(const {} of [1,2])count++;for(let {} in null)count++;count===2");
    check(
        "let log=[],error={},x=9,result,inner={next(){return{done:false,value:undefined};},return(){log.push('inner');throw 8;},[Symbol.iterator](){return this;}},outer={next(){return{done:false,value:inner};},return(){log.push('outer');throw 7;},[Symbol.iterator](){return this;}};function fail(){throw error;}try{for(const [x=fail()] of outer) ;}catch(e){result=e===error;}result && x===9 && log.join(',')==='inner,outer'",
    );
    check(
        "let closed=0,outer={next(){return{done:false,value:[7]};},return(){closed++;return{};},[Symbol.iterator](){return this;}};for(let [x] of outer)break;closed===1 && typeof x==='undefined'",
    );
}

#[test]
fn failed_initialization_retains_partial_bindings_and_captured_tdz_across_gc() {
    let mut realm = Realm::default();
    realm
        .eval("let saved;try{let [x=(saved=()=>y,1),y=missing]=[];}catch{}")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("let result;try{saved();}catch(e){result=e instanceof ReferenceError;}result"),
        Ok(Value::Boolean(true))
    );
    realm
        .eval("let fs=[];for(const {x} of [{x:{value:1}},{x:{value:2}}])fs.push(()=>x.value);")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("fs[0]()+fs[1]()"), Ok(Value::Number(3.0)));
    let mut realm = Realm::default();
    realm.eval("let saved;").unwrap();
    assert!(matches!(
        realm.eval("let [x=(saved=()=>x,1),y=missing]=[];"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert_eq!(realm.eval("x"), Ok(Value::Number(1.0)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("saved()"), Ok(Value::Number(1.0)));
    assert!(matches!(
        realm.eval("y"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
}

#[test]
fn early_global_conflicts_prevent_side_effects_and_flat_patterns_have_no_default_quota() {
    let mut realm = Realm::default();
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval("flag=9;let {undefined}={undefined:1};"),
        Err(Error::Exception {
            kind: ExceptionKind::SyntaxError,
            ..
        })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let names = (0..12000)
        .map(|i| format!("p{i}"))
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        realm.eval(&format!("let {{{names}}}={{}};p11999===undefined")),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn opted_in_host_aborts_restore_loop_scopes_and_skip_javascript_cleanup() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0,closed=0,inner={next(){return{done:false,value:1};},return(){closed++;return{};},[Symbol.iterator](){return this;}},outer={next(){return{done:false,value:inner};},return(){closed++;return{};},[Symbol.iterator](){return this;}};").unwrap();
    assert!(matches!(
        realm.eval("try{for(let [...rest] of outer)flag=3;}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && closed===0 && typeof rest==='undefined'"),
        Ok(Value::Boolean(true))
    );
}
