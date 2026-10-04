//! Reflect descriptors and complete own-key lists for supported objects.

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn object_targets_fail_before_keys_and_definitions_convert_key_before_attributes() {
    for method in ["defineProperty", "getOwnPropertyDescriptor", "ownKeys"] {
        for target in ["undefined", "null", "true", "1", "'s'", "1n", "Symbol()"] {
            check(&format!(
                "let reads=0,caught=false,key={{toString(){{reads++;throw 7;}}}},attrs={{get enumerable(){{reads++;throw 8;}}}};try{{Reflect.{method}({target},key,attrs);}}catch(e){{caught=e instanceof TypeError;}}caught && reads===0"
            ));
        }
    }
    check(
        "let log='',key={[Symbol.toPrimitive](hint){if(hint!=='string')throw 7;log+='k';return 'x';}},attrs={get enumerable(){log+='e';return true;},get configurable(){log+='c';return true;},get value(){log+='v';return 7;},get writable(){log+='w';return true;}},o={};Reflect.defineProperty(o,key,attrs)===true && log==='kecvw' && o.x===7",
    );
    check(
        "let reads=0,caught=false;try{Reflect.defineProperty({},{toString(){throw 7;}},{get enumerable(){reads++;throw 8;}});}catch(e){caught=e===7;}caught && reads===0",
    );
    check(
        "let reads=0,caught=false;try{Reflect.defineProperty({},{toString(){reads++;return 'x';}},undefined);}catch(e){caught=e instanceof TypeError;}caught && reads===1",
    );
}

#[test]
fn define_returns_boolean_and_descriptor_conversion_still_throws() {
    check(
        "let o={};Reflect.defineProperty(o,'x',{value:7})===true && Reflect.defineProperty(o,'x',{value:7})===true && Reflect.defineProperty(o,'x',{value:8})===false && o.x===7 && Reflect.defineProperty(o,'x',{writable:true})===false",
    );
    check(
        "let o={};Object.preventExtensions(o);Reflect.defineProperty(o,'x',{})===false && !Object.hasOwn(o,'x')",
    );
    check(
        "let o={};Reflect.defineProperty(o,'x',{value:NaN});Reflect.defineProperty(o,'x',{value:NaN})===true && Reflect.defineProperty(o,'x',{value:0})===false",
    );
    check(
        "let o={};Reflect.defineProperty(o,'x',{value:-0});Reflect.defineProperty(o,'x',{value:0})===false && 1/o.x===-Infinity",
    );
    check(
        "let calls=0,o={};Object.defineProperty(Object.prototype,'x',{set(){calls++;},configurable:true});Reflect.defineProperty(o,'x',{value:7,writable:true,enumerable:true,configurable:true}) && o.x===7 && calls===0",
    );
    check(
        "let o={},caught=false;try{Reflect.defineProperty(o,'x',{value:1,get(){}});}catch(e){caught=e instanceof TypeError;}caught && !Object.hasOwn(o,'x')",
    );
    check(
        "let reads=0,caught=false;try{Reflect.defineProperty({},'x',{get:1,get set(){reads++;throw 7;}});}catch(e){caught=e instanceof TypeError;}caught && reads===0",
    );
    check(
        "let o={},caught=false;try{Reflect.defineProperty(o,'x',{get enumerable(){throw 7;}});}catch(e){caught=e===7;}caught && !Object.hasOwn(o,'x')",
    );
}

#[test]
fn array_and_string_definitions_keep_internal_boolean_and_abrupt_results() {
    check(
        "let a=[1,2,3];Object.defineProperty(a,'1',{configurable:false});Reflect.defineProperty(a,'length',{value:0,writable:false})===false && a.length===2 && a[1]===2 && !Object.hasOwn(a,2) && Object.getOwnPropertyDescriptor(a,'length').writable===false",
    );
    check(
        "let calls=0,a=[1,2,3],v={valueOf(){calls++;return 2;}};Reflect.defineProperty(a,'length',{value:v})===true && calls===2 && a.length===2",
    );
    check(
        "let caught=false,a=[];try{Reflect.defineProperty(a,'length',{value:1.5});}catch(e){caught=e instanceof RangeError;}caught && a.length===0",
    );
    check(
        "let s=Object('ab');Reflect.defineProperty(s,0,{value:'a'})===true && Reflect.defineProperty(s,0,{value:'x'})===false && Reflect.defineProperty(s,'length',{value:3})===false && s[0]==='a' && s.length===2",
    );
}

#[test]
fn own_descriptors_are_fresh_and_never_invoke_getters() {
    check(
        "let o={x:7},d=Reflect.getOwnPropertyDescriptor(o,'x'),e=Reflect.getOwnPropertyDescriptor(o,'x');d!==e && Object.getPrototypeOf(d)===Object.prototype && Object.keys(d).join(',')==='value,writable,enumerable,configurable' && d.value===7 && d.writable && d.enumerable && d.configurable && (d.value=8)===8 && o.x===7",
    );
    check(
        "let reads=0,get=function(){reads++;throw 7;},o={};Object.defineProperty(o,'x',{get,configurable:true});let d=Reflect.getOwnPropertyDescriptor(o,'x');Object.keys(d).join(',')==='get,set,enumerable,configurable' && d.get===get && d.set===undefined && !d.enumerable && d.configurable && reads===0",
    );
    check(
        "let s=Symbol(),o=Object.create({x:1});o[s]=2;Reflect.getOwnPropertyDescriptor(o,'x')===undefined && Reflect.getOwnPropertyDescriptor(o,'missing')===undefined && Reflect.getOwnPropertyDescriptor(o,s).value===2",
    );
    check(
        "function f(a){a=7;let d=Reflect.getOwnPropertyDescriptor(arguments,0);return d.value===7 && d.writable && d.enumerable && d.configurable;}f(1)",
    );
}

#[test]
fn own_keys_include_all_key_types_in_property_order_without_values_or_species() {
    check(
        "let reads=0,s=Symbol(),t=Symbol(),p={p:1},o=Object.create(p);o.b=1;o[s]=2;o[2]=3;o.a=4;o[1]=5;o[t]=6;Object.defineProperty(o,'z',{get(){reads++;throw 7;}});let keys=Reflect.ownKeys(o);keys.length===7 && keys[0]==='1' && keys[1]==='2' && keys[2]==='b' && keys[3]==='a' && keys[4]==='z' && keys[5]===s && keys[6]===t && reads===0",
    );
    check(
        "let o={b:1,a:2};delete o.b;o.b=3;Reflect.ownKeys(o).join(',')==='a,b' && Reflect.ownKeys([,2]).join(',')==='1,length' && Reflect.ownKeys(Object('ab')).join(',')==='0,1,length'",
    );
    check(
        "let o={x:1},a=Reflect.ownKeys(o);delete o.x;o.y=2;a.join(',')==='x' && Reflect.ownKeys(o).join(',')==='y'",
    );
    check(
        "let calls=0;Object.defineProperty(Array.prototype,'0',{set(){calls++;},configurable:true});Array.prototype.constructor={get [Symbol.species](){throw 7;}};let keys=Reflect.ownKeys({x:1}),d=Object.getOwnPropertyDescriptor(keys,'0');keys[0]==='x' && Array.isArray(keys) && Object.getPrototypeOf(keys)===Array.prototype && d.writable && d.enumerable && d.configurable && calls===0",
    );
    check("let a=Array.from({length:4000});Reflect.ownKeys(a).length===4001");
}

#[test]
fn metadata_and_retained_intrinsics_survive_public_deletion() {
    for (method, length) in [
        ("defineProperty", 3),
        ("getOwnPropertyDescriptor", 2),
        ("ownKeys", 1),
    ] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Reflect,'{method}'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='{method}' && !n.writable && !n.enumerable && n.configurable && l.value==={length} && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')"
        ));
        check(&format!(
            "let caught=false;try{{new Reflect.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let define=Reflect.defineProperty,descriptor=Reflect.getOwnPropertyDescriptor,keys=Reflect.ownKeys;delete Reflect.defineProperty;delete Reflect.getOwnPropertyDescriptor;delete Reflect.ownKeys;delete globalThis.Reflect").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(realm.eval("let o={};define(o,'x',{value:7}) && descriptor(o,'x').value===7 && keys(o).join(',')==='x'"),Ok(Value::Boolean(true)));
}

#[test]
fn incomplete_intrinsics_and_host_gaps_skip_pending_handlers() {
    for operation in [
        "Reflect.ownKeys(String.prototype)",
        "Reflect.getOwnPropertyDescriptor(String.prototype,'normalize')",
        "Reflect.defineProperty(String.prototype,'normalize',{value:7})",
        "Reflect.defineProperty({},'x',{get value(){Proxy;}})",
    ] {
        let mut realm = Realm::default();
        realm.eval("let flag=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{{operation};}}catch{{flag=1;}}finally{{flag=2;}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{operation}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
