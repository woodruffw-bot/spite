//! JSON serialization hooks, key snapshots, quoting, indentation, and cycles.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn primitives_omissions_and_well_formed_utf16_quoting_are_exact() {
    check(
        "JSON.stringify(null)==='null' && JSON.stringify(true)==='true' && JSON.stringify(false)==='false' && JSON.stringify(-0)==='0' && JSON.stringify(NaN)==='null' && JSON.stringify(Infinity)==='null' && JSON.stringify(-Infinity)==='null' && JSON.stringify(undefined)===undefined && JSON.stringify(Symbol())===undefined && JSON.stringify(()=>{})===undefined",
    );
    check(
        r#"JSON.stringify('\u0000\b\t\n\f\r"\\/')==='"\\u0000\\b\\t\\n\\f\\r\\"\\\\/"' && JSON.stringify('\uD800x\uDC00')==='"\\ud800x\\udc00"' && JSON.stringify('\uD800\uDC00')==='"\uD800\uDC00"' && JSON.stringify('\u2028\u2029')==='"\u2028\u2029"'"#,
    );
    check(
        "JSON.stringify([undefined,Symbol(),()=>{},NaN,Infinity])==='[null,null,null,null,null]' && JSON.stringify({a:undefined,b:Symbol(),c(){},d:null})==='{\"d\":null}'",
    );
}

#[test]
fn object_keys_and_array_lengths_are_snapshotted_but_values_are_read_live() {
    check(
        r#"let value={z:1,2:2,1:1,get a(){delete this.b;this.c=4;return 3;},b:2};value[Symbol()]=5;Object.defineProperty(value,'hidden',{value:6});JSON.stringify(value)==='{"1":1,"2":2,"z":1,"a":3}'"#,
    );
    check(
        r#"let value=[1,2,3];Object.defineProperty(value,'0',{get(){value.length=1;value[5]=9;return 1;}});JSON.stringify(value)==='[1,null,null]' && value.length===6"#,
    );
    check("Array.prototype[1]=8;let value=[1,,3];value.extra=9;JSON.stringify(value)==='[1,8,3]'");
    check(
        r#"let trace='',value={get a(){trace+='a';return 1;},get b(){trace+='b';return 2;}};JSON.stringify(value,function(k,v){trace+='r'+k;return v;})==='{"a":1,"b":2}' && trace==='rara'+'brb'"#,
    );
}

#[test]
fn tojson_then_replacer_then_wrapper_conversion_follow_the_spec_order() {
    check(
        r#"let trace='',value={toJSON(k){trace+='to:'+k+';';return new Number(2);}},replacer=function(k,v){trace+='replace:'+k+';';let result=new String('yes');result[Symbol.toPrimitive]=function(hint){trace+=hint;return 'done';};return result;};JSON.stringify(value,replacer)==='"done"' && trace==='to:;replace:;string'"#,
    );
    check(
        "let trace='',f=function(){};f.toJSON=function(k){trace+=k;return 7;};JSON.stringify({x:f})==='{\"x\":7}' && trace==='x'",
    );
    check(
        "let calls=0,value={toJSON(k){if(this!==value || arguments.length!==1 || k!=='')throw 7;calls++;return {a:1};}};JSON.stringify(value)==='{\"a\":1}' && calls===1",
    );
    check(
        "let called=false,result=new Boolean(false);result.valueOf=function(){called=true;return true;};JSON.stringify(result)==='false' && !called",
    );
    check(
        "let called=false,result=new Number(3);result.valueOf=function(){called=true;return 4;};JSON.stringify(result)==='4' && called",
    );
    check(
        "JSON.stringify(Object(Symbol()))==='{}' && JSON.stringify({toJSON:3})==='{\"toJSON\":3}'",
    );
}

#[test]
fn replacer_callback_holders_root_wrapper_and_returned_values_are_preserved() {
    check(
        r#"let root={a:[1,2]},trace=[],wrapper;let text=JSON.stringify(root,function(k,v){if(arguments.length!==2)throw 7;trace.push(k);if(k===''){wrapper=this;if(this['']!==root)throw 8;}if(k==='a' && this!==root)throw 9;if(k==='0'){if(this!==root.a)throw 10;return undefined;}return v;});text==='{"a":[null,2]}' && trace.join(',')===',a,0,1' && Object.getPrototypeOf(wrapper)===Object.prototype && Object.keys(wrapper).join(',')===''"#,
    );
    check(
        "JSON.stringify({a:1,b:2},function(k,v){return k==='a'?undefined:v;})==='{\"b\":2}' && JSON.stringify(1,()=>undefined)===undefined",
    );
    check(
        "Object.defineProperty(Object.prototype,'',{set(){throw 7;}});JSON.stringify(1,(k,v)=>v)==='1'",
    );
    check(
        "JSON.stringify({a:1},function(k,v){if(k==='a')return {toJSON(){throw 7;},x:2};return v;})==='{\"a\":{\"x\":2}}'",
    );
}

#[test]
fn replacer_property_lists_convert_deduplicate_and_include_inherited_properties() {
    check(
        r#"let object={a:1,b:2,2:3};Object.defineProperty(object,'hidden',{value:4});Object.setPrototypeOf(object,{inherited:5});JSON.stringify(object,['b',2,'b','hidden','inherited',true,null,Symbol()])==='{"b":2,"2":3,"hidden":4,"inherited":5}' && JSON.stringify(object,[])==='{}'"#,
    );
    check(
        r#"let trace='',item=new Number(1);item.toString=function(){trace+='convert';return 'a';};let list=[item,'a',new String('b')];let gap=new Number(2);gap.valueOf=function(){trace+='space';return 2;};JSON.stringify({a:1,b:2},list,gap)==='{\n  "a": 1,\n  "b": 2\n}' && trace==='convertspace'"#,
    );
    check(
        "let list=['a','b'];Object.defineProperty(list,'0',{get(){list.length=1;return 'a';}});JSON.stringify({a:1,b:2},list)==='{\"a\":1}'",
    );
    check(
        "JSON.stringify([1,2],[])==='[1,2]' && JSON.stringify({a:{a:1,b:2},b:3},['a'])==='{\"a\":{\"a\":1}}'",
    );
    check(
        "let list=[new Boolean(true),Object(Symbol()),Object(1n),{},undefined];JSON.stringify({a:1},list)==='{}'",
    );
}

#[test]
fn indentation_clamps_numbers_and_truncates_string_code_units() {
    check(
        r#"JSON.stringify({a:[1,{}]},null,2)==='{\n  "a": [\n    1,\n    {}\n  ]\n}' && JSON.stringify([],null,2)==='[]' && JSON.stringify({},null,2)==='{}' && JSON.stringify({a:1},null,1.9)==='{\n "a": 1\n}'"#,
    );
    check(
        r#"JSON.stringify([1],null,Infinity)==='[\n          1\n]' && JSON.stringify([1],null,-Infinity)==='[1]' && JSON.stringify([1],null,NaN)==='[1]'"#,
    );
    check(
        r#"JSON.stringify([1],null,'123456789\uD800\uDC00')==='[\n123456789\uD8001\n]' && JSON.stringify([1],null,new String('x'))==='[\nx1\n]'"#,
    );
    check(
        "let value={valueOf(){throw 7;},toString(){throw 8;}};JSON.stringify([1],null,value)==='[1]'",
    );
}

#[test]
fn bigint_hooks_raw_embedding_and_branded_values_obey_hook_order() {
    for source in [
        "JSON.stringify(1n)",
        "JSON.stringify(Object(1n))",
        "JSON.stringify([1n])",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{source}"
        );
    }
    check(
        "BigInt.prototype.toJSON=function(k){'use strict';if(typeof this!=='bigint' || k!=='x')throw 7;return JSON.rawJSON(this);};JSON.stringify({x:9007199254740993n})==='{\"x\":9007199254740993}'",
    );
    check(
        "JSON.stringify({x:9007199254740993n},(k,v)=>typeof v==='bigint'?JSON.rawJSON(v):v)==='{\"x\":9007199254740993}'",
    );
    check(
        r#"JSON.stringify([JSON.rawJSON('1E+02'),JSON.rawJSON('1e400'),JSON.rawJSON('"\\uD800"')])==='[1E+02,1e400,"\\uD800"]'"#,
    );
    check(
        "let raw=JSON.rawJSON('1'),called=0;JSON.stringify(raw,function(k,v){called++;return 'x';})==='\"x\"' && called===1 && JSON.stringify({...raw})==='{\"rawJSON\":\"1\"}'",
    );
}

#[test]
fn ancestor_cycle_detection_allows_aliases_and_hooks_can_break_cycles() {
    for source in [
        "let a=[];a[0]=a;JSON.stringify(a)",
        "let a={};a.x=a;JSON.stringify(a)",
        "let a={},b={a};a.b=b;JSON.stringify(a)",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{source}"
        );
    }
    check("let shared={x:1};JSON.stringify([shared,shared])==='[{\"x\":1},{\"x\":1}]'");
    check(
        "let value={};value.self=value;JSON.stringify(value,(k,v)=>k==='self'?null:v)==='{\"self\":null}'",
    );
    check(
        "let value={};value.self=value;value.toJSON=function(){return 7;};JSON.stringify(value)==='7'",
    );
}

#[test]
fn abrupt_getters_hooks_replacers_and_nested_serialization_keep_identity() {
    for body in [
        "let value={get x(){throw marker;}};JSON.stringify(value)",
        "JSON.stringify({get toJSON(){throw marker;}})",
        "JSON.stringify({toJSON(){throw marker;}})",
        "JSON.stringify(1,()=>{throw marker;})",
    ] {
        check(&format!(
            "let marker={{}},caught=false;try{{{body};}}catch(e){{caught=e===marker;}}caught"
        ));
    }
    check(
        "let number=new Number(1),marker={},caught=false;number.toString=function(){throw marker;};try{JSON.stringify(1,[number]);}catch(e){caught=e===marker;}caught",
    );
    check(
        "let trace='',value={get a(){trace+='a';throw 7;},get b(){trace+='b';return 2;}};let caught=false;try{JSON.stringify(value);}catch(e){caught=e===7;}caught && trace==='a'",
    );
    check(
        "JSON.stringify({a:1},function(k,v){return k==='a'?JSON.stringify([v]):v;})==='{\"a\":\"[1]\"}'",
    );
}

#[test]
fn complete_metadata_reflection_and_saved_functions_survive_collection() {
    check(
        "JSON.stringify.name==='stringify' && JSON.stringify.length===3 && Object.getPrototypeOf(JSON.stringify)===Function.prototype && !Object.hasOwn(JSON.stringify,'prototype') && Object.getOwnPropertyNames(JSON).length===4 && Object.getOwnPropertySymbols(JSON).length===1 && Object.assign({},JSON).parse===undefined && Object.keys(JSON).length===0 && Object.freeze(JSON)===JSON && Object.isFrozen(JSON)",
    );
    assert!(matches!(
        Realm::default().eval("new JSON.stringify(1)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm
        .eval("let stringify=JSON.stringify;delete JSON.stringify;delete globalThis.JSON;")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("stringify({x:1})==='{\"x\":1}'"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn deep_default_serialization_is_iterative_and_host_work_abort_skips_handlers() {
    check(
        "let text='['.repeat(10000)+'0'+']'.repeat(10000);JSON.stringify(JSON.parse(text))===text",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval("try{JSON.stringify(Array(20000));}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_string_units: Some(8),
        ..Limits::default()
    });
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval("try{JSON.stringify([1,2,3,4,5]);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
