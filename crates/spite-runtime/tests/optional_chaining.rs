//! Lazy nullish checks and preserved references (13.3.10, 6.2.5).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn only_nullish_bases_short_circuit_and_ungrouped_suffixes_skip_keys_and_arguments() {
    for value in ["null", "undefined"] {
        check(&format!(
            "let x={value},count=0,key={{[Symbol.toPrimitive](){{count++;throw 7;}}}};let a=x?.[(count++,key)].b(count++).c?.(count++),b=x?.(count++).c; a===undefined && b===undefined && count===0"
        ));
    }
    check(
        "false?.toString()==='false' && (0)?.valueOf()===0 && ''?.length===0 && 0n?.valueOf()===0n && [7]?.[0]===7 && Symbol('x')?.description==='x'",
    );
    check(
        "let calls=0,o={valueOf(){calls++;throw 7;},toString(){calls++;throw 8;},x:9};o?.x===9 && calls===0",
    );
    check(
        "let gets=0,steps=0,o={get x(){gets++;return undefined;}};let r=o?.x?.[(steps++,7)].z(steps++);r===undefined && gets===1 && steps===0",
    );
}

#[test]
fn grouping_ends_short_circuiting_but_preserves_property_call_receivers() {
    check(
        "let n=0,caught=0;null?.m(n++);try{(null?.m)(n++);}catch(e){if(e instanceof TypeError)caught++;}try{(null?.x).y;}catch(e){if(e instanceof TypeError)caught++;}caught===2 && n===1",
    );
    for call in [
        "a?.m(7)",
        "a.m?.(7)",
        "a?.m?.(7)",
        "(a?.m)(7)",
        "(a?.m)?.(7)",
        "(a.m)?.(7)",
    ] {
        check(&format!(
            "let calls=0,a={{m:function(value){{'use strict';calls++;return this===a && value===7 && arguments.length===1;}}}};{call} && calls===1"
        ));
    }
    check(
        "let a={m:function(){'use strict';return this;}};(0,a?.m)()===undefined && (a?.m)`x`===a",
    );
    check("let context;function f(){'use strict';context=this;}f?.();context===undefined");
}

#[test]
fn getters_keys_and_calls_are_ordered_once_with_late_callable_checks() {
    check(
        "let log='',receiver,argc,o={get m(){log+='g';return function(value){'use strict';receiver=this;argc=arguments.length;log+='c';return value;};}},key={[Symbol.toPrimitive](hint){log+='k';if(hint!=='string')throw 7;return 'm';}};let result=o?.[(log+='e',key)]?.((log+='a',8));result===8 && log==='ekgac' && receiver===o && argc===1",
    );
    check(
        "let gets=0,steps=0,o={get m(){gets++;return null;}};o.m?.(steps++)===undefined && gets===1 && steps===0",
    );
    check(
        "let log='',o={m:7},caught=false;try{o?.m?.((log+='a',1));}catch(e){caught=e instanceof TypeError;}caught && log==='a'",
    );
    check(
        "let sentinel={},log='',o={m:7},caught=false;try{o?.m((()=>{log+='a';throw sentinel;})(),(log+='b',2));}catch(e){caught=e===sentinel;}caught && log==='a'",
    );
    check("let original=function(){return 7;},o={m:original};o?.m?.((o.m=()=>8))===7 && o.m()===8");
    check("let count=0;function f(){count++;return null;}f()?.x===undefined && count===1");
}

#[test]
fn only_explicit_optional_steps_guard_later_nullish_values() {
    check(
        "let o={},caught=false;try{o?.x.y;}catch(e){caught=e instanceof TypeError;}caught && o?.x?.y===undefined",
    );
    check(
        "let log='',key={[Symbol.toPrimitive](){log+='k';return 'x';}},caught=false;try{({x:null})?.x[(log+='e',key)];}catch(e){caught=e instanceof TypeError;}caught && log==='e'",
    );
    check(
        "let log='',target={},o={get a(){log+='a';return target;}},key={[Symbol.toPrimitive](){log+='k';return 'b';}};(o?.a)[(log+='e',key)]=(log+='r',7);log==='aerk' && target.b===7",
    );
    check("(true?.30:false)===0.3");
}

#[test]
fn deletion_keeps_the_final_reference_and_never_reads_its_getter() {
    check(
        "let gets=0,o={get x(){gets++;throw 7;}};delete o?.x && gets===0 && !Object.hasOwn(o,'x')",
    );
    check("let gets=0,o={get a(){gets++;return {get x(){throw 7;}};}};delete o?.a.x && gets===1");
    check(
        "let n=0,key={[Symbol.toPrimitive](){n++;throw 7;}};delete null?.[key] && delete undefined?.x.y && n===0",
    );
    check(
        "let symbol=Symbol(),n=0,key={[Symbol.toPrimitive](){n++;return symbol;}},o={[symbol]:7};delete o?.[key] && n===1 && !Object.hasOwn(o,symbol)",
    );
    check("let o=Object.freeze({x:7});(delete o?.x)===false && o.x===7");
    check(
        "'use strict';let o=Object.freeze({x:7}),caught=false;try{delete o?.x;}catch(e){caught=e instanceof TypeError;}caught && delete null?.x",
    );
    check("let calls=0,o={f(){calls++;return {x:7};}};delete o?.f() && calls===1");
}

#[test]
fn unresolvable_and_uninitialized_bases_throw_and_typeof_does_not_hide_them() {
    for source in ["missing?.x", "typeof missing?.x", "let x=x?.a;"] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::ReferenceError,
                    ..
                })
            ),
            "{source}"
        );
    }
    check("let x;typeof x?.value==='undefined'");
    check(
        "let sentinel={},o={get x(){throw sentinel;}},flag=0,caught=false;try{o?.x;}catch(e){caught=e===sentinel;}finally{flag=7;}caught && flag===7",
    );
}

#[test]
fn primitive_receivers_spread_newtarget_and_grouped_construction_follow_normal_calls() {
    check(
        "Object.defineProperty(Number.prototype,'f',{value:function(){'use strict';return this;}});(7)?.f?.()===7",
    );
    check(
        "let a={f:function(x,y){'use strict';return this===a && x===7 && y===8 && arguments.length===2;}};a?.f?.(...[7,8])",
    );
    check(
        "let n=0,context;function F(){new.target?.();}function G(){n++;context=this;}Reflect.construct(F,[],G);n===1 && context===globalThis",
    );
    check("let o={C:function(x){this.x=x;}},a=new (o?.C)(7);a.x===7 && a instanceof o.C");
    check("let n=0;function F(){n++;this.x=7;}new F()?.x===7 && n===1");
}

#[test]
fn flat_default_chains_and_captured_optional_syntax_survive_collection() {
    check(&format!(
        "let x={{}};x.self=x;let y=x?.self{};y===x",
        ".self".repeat(1000)
    ));
    let mut realm = Realm::default();
    realm
        .eval("let object={x:{value:7}},read=()=>object?.x?.value;")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("read?.()"), Ok(Value::Number(7.0)));
}

#[test]
fn opted_in_host_limits_and_unsupported_features_skip_javascript_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm
        .eval("let flag=0,o={get x(){while(true){};}};")
        .unwrap();
    assert!(matches!(
        realm.eval("try{o?.x;}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::default();
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval("try{Function?.((flag=7,'class C{}'));}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(7.0)));
    realm.reserve_unsupported_global("missingHost");
    assert!(matches!(
        realm.eval("try{missingHost?.();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(7.0)));
}
