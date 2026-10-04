//! JSON.parse reviver traversal and immutable source snapshots, ECMA-262 25.5.1.

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn postorder_key_order_root_holder_and_callback_arguments_are_standard() {
    check(
        r#"let trace=[],holders=[],contexts=[];let value=JSON.parse('{"z":[],"2":2,"a":{"x":1},"1":1}',function(k,v,c){if(arguments.length!==3)throw 7;if(Object.getPrototypeOf(c)!==Object.prototype)throw 8;trace.push(k);holders.push(this);contexts.push(c);return v;});trace.join(',')==='1,2,z,x,a,' && holders[0]===value && holders[3]===value.a && holders[5]['']===value && Object.keys(holders[5]).join(',')==='' && contexts[0]!==contexts[1] && !Object.hasOwn(contexts[2],'source') && !Object.hasOwn(contexts[4],'source') && !Object.hasOwn(contexts[5],'source')"#,
    );
    check(
        "JSON.parse('1',()=>undefined)===undefined && JSON.parse('1',()=>7)===7 && JSON.parse('1',()=>2n)===2n && typeof JSON.parse('null',()=>Symbol())==='symbol'",
    );
    check(
        "let count=0,f=function(k,v,c){if(this.x!==7 || arguments.length!==4 || k!=='bound' || v!=='')throw 8;count++;return c;}.bind({x:7},'bound');JSON.parse('1',f)===1 && count===1",
    );
}

#[test]
fn primitive_context_sources_preserve_utf16_exact_lexemes_and_descriptors() {
    check(
        r#"let sources=[],contexts=[];JSON.parse(' [ -0, 1E+02, "\\u0061", true, null, 1e400 ] ',function(k,v,c){if(k!==''){sources.push(c.source);contexts.push(c);}return v;});let d=Object.getOwnPropertyDescriptor(contexts[0],'source');sources.join('|')==='-0|1E+02|"\\u0061"|true|null|1e400' && d.writable && d.enumerable && d.configurable && Object.keys(contexts[0]).join(',')==='source'"#,
    );
    check(
        r#"let source;JSON.parse('"'+ '\uD800' +'"',function(k,v,c){source=c.source;return v;});source==='"'+ '\uD800' +'"'"#,
    );
    check(
        r#"let seen=[];JSON.parse('{"a":1,"\\u0061":2,"__proto__":3,"__proto__":4}',function(k,v,c){if(k)seen.push(k+':'+c.source);return v;});seen.join(',')==='a:2,__proto__:4'"#,
    );
}

#[test]
fn changed_values_and_replaced_containers_do_not_reuse_original_sources() {
    check(
        r#"let seen=[];JSON.parse('{"a":1,"b":2,"c":3,"d":4,"e":{"x":5}}',function(k,v,c){if(k==='a'){this.b=9;delete this.c;this.d=4;this.e={x:5};}if(k)seen.push(k+':'+(Object.hasOwn(c,'source')?c.source:'none'));return v;});seen.join(',')==='a:1,b:none,c:none,d:4,x:none,e:none'"#,
    );
    check(
        r#"let seen=[];JSON.parse('[-0,1,2]',function(k,v,c){if(k==='0'){this[1]=-0;this[2]=2;}if(k!=='')seen.push(k+':'+(Object.hasOwn(c,'source')?c.source:'none'));return v;});seen.join(',')==='0:-0,1:none,2:2'"#,
    );
    check(
        r#"let seen=[];JSON.parse('[0,1]',function(k,v,c){if(k==='0')Object.defineProperty(this,'1',{get(){return 1;},configurable:true});if(k==='1')seen.push(c.source);return v;});seen[0]==='1'"#,
    );
}

#[test]
fn keys_and_array_length_are_snapshotted_before_child_callbacks() {
    check(
        r#"let seen=[],value=JSON.parse('{"a":1,"b":2,"c":3}',function(k,v,c){if(k==='a'){this.added=4;delete this.b;Object.defineProperty(this,'c',{enumerable:false});}seen.push(k);return v;});seen.join(',')==='a,b,c,' && !Object.hasOwn(value,'b') && value.added===4 && Object.getOwnPropertyDescriptor(value,'c').enumerable"#,
    );
    check(
        r#"let seen=[],value=JSON.parse('[1,2,3]',function(k,v,c){if(k==='0'){this.length=1;this[5]=9;}seen.push(k);return v;});seen.join(',')==='0,1,2,' && value.length===6 && value[5]===9 && !Object.hasOwn(value,'1') && !Object.hasOwn(value,'2')"#,
    );
    check(
        r#"Array.prototype[1]=8;let source='unset',value=JSON.parse('[1,2]',function(k,v,c){if(k==='0')delete this[1];if(k==='1'){source=c.source;return v+1;}return v;});value[1]===9 && source===undefined && Object.hasOwn(value,'1')"#,
    );
    check(
        r#"let seen=[],value=JSON.parse('[0,1]',function(k,v,c){if(k==='0')this[1]={a:2,b:3};seen.push(k);return v;});seen.join(',')==='0,a,b,1,' && value[1].b===3"#,
    );
}

#[test]
fn return_values_use_own_data_properties_and_ignore_rejected_updates() {
    check(
        r#"let sets=0,value=JSON.parse('{"a":1,"b":2,"c":3,"d":4}',function(k,v,c){if(k==='a'){Object.defineProperty(this,'b',{value:2,writable:false,configurable:false});Object.defineProperty(this,'c',{value:3,writable:false,configurable:false});Object.defineProperty(this,'d',{set(){sets++;},configurable:true});}if(k==='b')return 9;if(k==='c')return undefined;if(k==='d')return 8;return v;});let d=Object.getOwnPropertyDescriptor(value,'d');value.b===2 && value.c===3 && value.d===8 && sets===0 && d.writable && d.enumerable && d.configurable"#,
    );
    check(
        r#"let value=JSON.parse('[1,2,3]',function(k,v){if(k==='0')Object.freeze(this);if(k==='1')return undefined;return 9;});value===9"#,
    );
    check(
        r#"let value=JSON.parse('{"__proto__":1,"x":2}',function(k,v){if(k==='__proto__')return {safe:true};return v;});Object.getPrototypeOf(value)===Object.prototype && Object.hasOwn(value,'__proto__') && value.__proto__.safe"#,
    );
}

#[test]
fn callback_and_getter_abrupt_completions_keep_identity_and_stop_traversal() {
    check(
        r#"let marker={},trace='',caught=false;try{JSON.parse('[1,2]',function(k,v){trace+=k;if(k==='0')throw marker;return v;});}catch(e){caught=e===marker;}caught && trace==='0'"#,
    );
    check(
        r#"let marker={},trace='',caught=false;try{JSON.parse('{"a":1,"b":2}',function(k,v){trace+=k;if(k==='a')Object.defineProperty(this,'b',{get(){throw marker;},configurable:true});return v;});}catch(e){caught=e===marker;}caught && trace==='a'"#,
    );
    check(
        r#"let trace='',f=function(k,v){trace+=k;if(k==='0')JSON.parse('1',()=>{throw 7;});return v;};let caught=false;try{JSON.parse('[1,2]',f);}catch(e){caught=e===7;}caught && trace==='0'"#,
    );
}

#[test]
fn deep_traversal_uses_no_native_recursion_and_opted_in_abort_stops_cycles() {
    check(
        "let count=0,value=JSON.parse('['.repeat(10000)+'0'+']'.repeat(10000),function(k,v,c){count++;return v;});for(let i=0;i<10000;i++)value=value[0];count===10001 && value===0",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(10000),
        ..Limits::default()
    });
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(realm.eval("try{JSON.parse('[0,1]',function(k,v){if(k==='0')this[1]=this;return v;});}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit { .. })));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
