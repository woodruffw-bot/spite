//! Spread arguments preserve call/construction ordering (13.3.8.1).

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn mixed_arguments_flatten_iterables_in_order_and_preserve_holes_as_undefined() {
    check(
        "function f(){return Array.from(arguments);}let a=f(1,...[2,,4],...'A\\uD834\\uDF06',...[],5);a.length===7 && a[0]===1 && a[1]===2 && a[2]===undefined && a[3]===4 && a[4]==='A' && a[5]==='\\uD834\\uDF06' && a[6]===5",
    );
    check("function f(){return arguments.length;}f(...[])===0 && f(1,...[],2)===2");
    check(
        "function f(){return arguments[0].name;}f(...[],()=>{})==='' && f(function(){},...[])===''",
    );
    check("function f(a){a=7;return arguments[0];}f(...[1])===7");
    check("function f(){return ((a,b)=>a+b)(...arguments);}f(1,2)===3");
}

#[test]
fn spread_finishes_before_later_arguments_and_invocation() {
    check(
        "let log='',count=0;function ordinary(n){log+='e'+n;return n;}let source={get [Symbol.iterator](){log+='g';return function(){log+='i';return {next(){log+='n';return {value:++count,done:count===3};}};};}};function f(){log+='f';return Array.from(arguments).join(',');}let result=f(ordinary(0),...source,ordinary(4));result==='0,1,2,4' && log==='e0ginnne4f'",
    );
    check(
        "let source=[1,2],count=0;Object.defineProperty(source,'0',{get(){if(count++===0)source.push(3);return 1;}});function f(){return Array.from(arguments).join(',');}f(...source)==='1,2,3'",
    );
}

#[test]
fn original_callee_and_member_receiver_survive_iterator_side_effects() {
    check(
        "let gets=0,o={n:7,get f(){gets++;return function(a){return this.n+a;};}},source={[Symbol.iterator](){Object.defineProperty(o,'f',{value:()=>99});return [1][Symbol.iterator]();}};o.f(...source)===8 && gets===1 && o.f()===99",
    );
    check(
        "function strict(){'use strict';return this;}let o={strict};o.strict(...[])===o && (0,o.strict)(...[])===undefined",
    );
    check(
        "let reads=0,source={get [Symbol.iterator](){reads++;return [][Symbol.iterator];}};let caught=false;try{unresolvable(...source);}catch(e){caught=e instanceof ReferenceError;}caught && reads===0",
    );
}

#[test]
fn construction_evaluates_spread_before_prototype_lookup_and_keeps_new_target() {
    check(
        "let prototype={},source={[Symbol.iterator](){F.prototype=prototype;return [1,2][Symbol.iterator]();}};function F(a,b){this.sum=a+b;this.target=new.target;this.count=arguments.length;}let a=new F(...source);a.sum===3 && a.target===F && a.count===2 && Object.getPrototypeOf(a)===prototype",
    );
    check(
        "function F(a,b){this.sum=a+b;}let Bound=F.bind(null,1);let a=new Bound(...[2]);a.sum===3 && a instanceof F && a instanceof Bound",
    );
    check("let a=new Array(...[1,2,3]);a.join(',')==='1,2,3' && Array.isArray(a)");
    check(
        "let steps=0,source={[Symbol.iterator](){return {next(){return {done:++steps===3,value:1};}};}};let caught=false;try{new BigInt(...source);}catch(e){caught=e instanceof TypeError;}caught && steps===3",
    );
}

#[test]
fn noncallable_callee_checks_follow_complete_argument_evaluation() {
    check(
        "let steps=0,later=0,f=7,source={[Symbol.iterator](){return {next(){return {done:++steps===3,value:1};}};}};let caught=false;try{f(...source,++later);}catch(e){caught=e instanceof TypeError;}caught && steps===3 && later===1",
    );
    check(
        "let f=7,caught=false;try{f(...{get [Symbol.iterator](){throw 8;}});}catch(e){caught=e===8;}caught",
    );
}

#[test]
fn bad_spread_protocols_throw_and_skip_later_arguments_without_cleanup() {
    for source in [
        "null",
        "undefined",
        "7",
        "true",
        "7n",
        "Symbol()",
        "{length:1,0:7}",
        "{[Symbol.iterator]:null}",
        "{[Symbol.iterator]:7}",
        "{[Symbol.iterator](){return 7;}}",
        "{[Symbol.iterator](){return {next:7};}}",
        "{[Symbol.iterator](){return {next(){return 7;}};}}",
    ] {
        check(&format!(
            "let later=0,called=0;function f(){{called++;}}let caught=false;try{{f(...({source}),++later);}}catch(e){{caught=e instanceof TypeError;}}caught && later===0 && called===0"
        ));
    }
    for body in [
        "throw 7;",
        "return {get done(){throw 7;}};",
        "return {get value(){throw 7;}};",
    ] {
        check(&format!(
            "let closed=0,later=0,source={{[Symbol.iterator](){{return {{next(){{{body}}},return(){{closed++;return {{}};}}}};}}}};function f(){{throw 9;}}let caught=false;try{{f(...source,++later);}}catch(e){{caught=e===7;}}caught && closed===0 && later===0"
        ));
    }
}

#[test]
fn explicit_argument_and_work_quotas_abort_without_cleanup_or_handlers() {
    for (limits, source) in [
        (
            Limits {
                max_arguments: Some(2),
                ..Limits::default()
            },
            "f(...source)",
        ),
        (
            Limits {
                max_steps: Some(5000),
                ..Limits::default()
            },
            "new f(...source)",
        ),
    ] {
        let mut realm = Realm::new(limits);
        realm.eval("let closed=0,flag=0,called=0,steps=0;function f(){called++;}let source={[Symbol.iterator](){return {next(){steps++;return {value:steps};},return(){closed++;return {};}};}}").unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try{{{source};}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Limit { .. })
        ));
        assert_eq!(
            realm.eval("closed===0 && flag===0 && called===0"),
            Ok(Value::Boolean(true))
        );
        if limits.max_arguments.is_some() {
            assert_eq!(realm.eval("steps"), Ok(Value::Number(3.0)));
        }
    }
}

#[test]
fn default_calls_accept_large_argument_lists_without_a_work_or_argument_cutoff() {
    check(
        "let steps=0,source={[Symbol.iterator](){return {next(){return {value:steps,done:++steps===4001};}};}},f=()=>7;f(...source)===7 && steps===4001",
    );
}
