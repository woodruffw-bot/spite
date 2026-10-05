//! Live with bindings, unscopables, call receivers, and restored scopes (14.11).

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn own_inherited_and_nonenumerable_properties_shadow_live_bindings() {
    check(
        "let x=1,y=2,o=Object.create({x:3});Object.defineProperty(o,'y',{value:4});let a,b;with(o){a=x;b=y;}a===3 && b===4 && x===1 && y===2",
    );
    check("let x=1,o={},a,b,c;with(o){a=x;o.x=2;b=x;delete o.x;c=x;}a===1 && b===2 && c===1");
    check(
        "let x=1,a={x:2},b={x:3},values=[];with(a){values.push(x);with(b){values.push(x);delete b.x;values.push(x);}values.push(x);}values.push(x);values.join(',')==='2,3,2,2,1'",
    );
    check(
        "let o={eval:function(){return 7;},Proxy:8},a,b;with(o){a=eval();b=Proxy;}a===7 && b===8",
    );
}

#[test]
fn unscopables_uses_live_truthiness_and_inherited_receivers() {
    check(
        "let x=1,o={x:2},u={x:true},a,b,c,d;o[Symbol.unscopables]=u;with(o){a=x;u.x=0;b=x;u.x={};c=x;delete u.x;d=x;}a===1 && b===2 && c===1 && d===2",
    );
    check(
        "let x=1,o=Object.create({x:2}),u=Object.create({x:true});Object.defineProperty(o,Symbol.unscopables,{get(){return this===o?u:null;}});let a;with(o){a=x;}a===1",
    );
    check(
        "let x=1,o={x:2},u={get x(){return this===u;}};o[Symbol.unscopables]=u;let a;with(o){a=x;}a===1",
    );
    check(
        "let x=1,o={x:2},results=[];for(let u of [null,undefined,false,7,'text',Symbol(),7n]){o[Symbol.unscopables]=u;with(o)results.push(x);}results.join(',')==='2,2,2,2,2,2,2'",
    );
    check("let absent=7,o={get [Symbol.unscopables](){throw 9;}},a;with(o){a=absent;}a===7");
    check(
        "let values=7,at=8,seen=[];with([1,2]){seen.push(typeof values);seen.push(at);}seen.join(',')==='number,8'",
    );
}

#[test]
fn lookup_getters_throw_and_typeof_resolves_only_once() {
    check(
        "let x=1,o={x:2,get [Symbol.unscopables](){throw 7;}},a;try{with(o)x;}catch(e){a=e;}a===7 && x===1",
    );
    check(
        "let o={x:2},u={get x(){throw 8;}},a;o[Symbol.unscopables]=u;try{with(o)typeof x;}catch(e){a=e;}a===8",
    );
    check(
        "let n=0,o={x:2,get [Symbol.unscopables](){n++;return {};}};let a;with(o){a=typeof (x);}a==='number' && n===1",
    );
    check(
        "let o={get [Symbol.unscopables](){throw 7;}},a;with(o){a=typeof missing;}a==='undefined'",
    );
    check(
        "let o={x:2},u={get x(){delete o.x;return false;}};o[Symbol.unscopables]=u;let x=7,a;with(o){a=x;}a===undefined && x===7",
    );
}

#[test]
fn assignments_and_deletions_keep_the_resolved_binding_identity() {
    check("let x=1,o={x:2};with(o){x=3;x+=4;x++;}o.x===8 && x===1");
    check("let x=1,o={x:2},u={};o[Symbol.unscopables]=u;with(o){x=(u.x=true,7);}o.x===7 && x===1");
    check("let x=1,o={x:2};with(o){x=(delete o.x,7);}o.x===7 && x===1");
    check("let x=1,o={x:2},a;with(o){a=delete x;}a && !('x' in o) && x===1");
    check(
        "let o={},a;Object.defineProperty(o,'x',{value:2});with(o){x=7;a=delete x;}o.x===2 && !a",
    );
    check(
        "let x=1,o={x:2,[Symbol.unscopables]:{x:true}},a;with(o){x=7;a=delete x;}x===7 && o.x===2 && !a",
    );
    check(
        "let x=1,o={x:2},u={get x(){delete o.x;return false;}},a;o[Symbol.unscopables]=u;with(o){a=delete x;}a && !('x' in o) && x===1",
    );
}

#[test]
fn identifier_calls_optional_calls_and_tags_use_the_binding_object() {
    check(
        "let o={f:function(){'use strict';return this;}},a,b,c,d;with(o){a=f();b=(f)();c=f?.();d=f`x`;}a===o && b===o && c===o && d===o",
    );
    check("let o={f:function(){'use strict';return this;}},a;with(o){a=(0,f)();}a===undefined");
    check("let o=Object.create({f:function(){'use strict';return this;}}),a;with(o){a=f();}a===o");
    check(
        "let o={f:function(){'use strict';return this;}},a;with(o){let f=function(){'use strict';return this;};a=f();}a===undefined",
    );
    check(
        "let f=function(){'use strict';return this;},o={f(){return 7;},[Symbol.unscopables]:{f:true}},a;with(o){a=f();}a===undefined",
    );
}

#[test]
fn closures_capture_object_bindings_while_dynamic_functions_use_global_scope() {
    check(
        "let x=1,o={x:2},f,g;with(o){f=function(){return x;};g=()=>x;}o.x=3;let a=f(),b=g();delete o.x;a===3 && b===3 && f()===1 && g()===1",
    );
    check("let x=1,o={x:2},f;with(o){f=Function('return x;');}f()===1");
    check(
        "let receiver={},o={},f;function C(){with(o){f=()=>[this,new.target];}}Reflect.construct(C,[],C);let a=f();a[0] instanceof C && a[1]===C",
    );
    check(
        "let o={x:2},f;with(o){f=function(){'use strict';return x;};}o[Symbol.unscopables]={get x(){delete o.x;return false;}};let a;try{f();}catch(e){a=e instanceof ReferenceError;}a===true",
    );
    check(
        "let o={x:2},f;with(o){f=function(){'use strict';x=(delete o.x,7);};}let a;try{f();}catch(e){a=e instanceof ReferenceError;}a===true && !('x' in o)",
    );
    check(
        "let o={},f;Object.defineProperty(o,'x',{value:2});with(o){f=function(){'use strict';x=7;};}let a;try{f();}catch(e){a=e instanceof TypeError;}a===true && o.x===2",
    );
}

#[test]
fn vars_hoist_and_blocks_keep_lexical_bindings_separate() {
    check(
        "let o={x:1},before=x;with(o){var x=2;var y=3;}before===undefined && x===undefined && globalThis.y===3 && o.x===2 && !('y' in o)",
    );
    check(
        "let x=1,o={x:2},a,b;with(o){let x=3;a=x;{let x=4;b=x;}}a===3 && b===4 && o.x===2 && x===1",
    );
    check("let o={x:2},f;with(o){function g(){return x;}f=g;}f()===2 && typeof g==='undefined'");
    check(
        "let o={x:2},f;function outer(){with(o){var x=3;}return x;}outer()===undefined && o.x===3",
    );
}

#[test]
fn object_conversion_and_all_completion_paths_restore_the_outer_scope() {
    check("let length=7,a,b;with('abc'){a=length;b=valueOf();}a===3 && b==='abc' && length===7");
    check(
        "let results=[];for(let x of [true,7,7n,Symbol('s')]){with(x){results.push(typeof valueOf());}}results.join(',')==='boolean,number,bigint,symbol'",
    );
    check("let n=0,o={[Symbol.toPrimitive](){n++;throw 7;}},x=1;with(o);n===0 && x===1");
    check(
        "let a=0;for(let x of [null,undefined]){try{with(x)a=9;}catch(e){if(e instanceof TypeError)a++;}}a===2",
    );
    check(
        "let x=1,o={x:2},a;function f(){with(o)return x;}a=f();let b;try{with(o)throw x;}catch(e){b=e;}a===2 && b===2 && x===1",
    );
    check(
        "let x=1,o={x:2},a=[];outer:for(let i=0;i<3;i++){with(o){if(i===0)continue outer;if(i===2)break outer;a.push(x);}a.push(x);}a.join(',')==='2,1' && x===1",
    );
    assert_eq!(Realm::default().eval("7;with({}){}"), Ok(Value::Undefined));
    assert_eq!(
        Realm::default().eval("7; a: { with({}) break a; }"),
        Ok(Value::Undefined)
    );
}

#[test]
fn captured_binding_objects_are_traced_and_host_aborts_restore_scopes() {
    let mut realm = Realm::default();
    realm
        .eval("let f;with({x:{value:7}}){f=function(){return x.value;};}")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f()"), Ok(Value::Number(7.0)));
    assert!(matches!(
        realm.eval("with({x:2})Function('class C{field;}');"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(
        realm.eval("typeof x"),
        Ok(Value::String("undefined".into()))
    );
    let mut limited = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    limited.eval("let x=1,flag=0;").unwrap();
    assert!(matches!(
        limited.eval("try{with({x:2})while(true){x++;}}catch(e){flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(limited.eval("x===1 && flag===0"), Ok(Value::Boolean(true)));
}
