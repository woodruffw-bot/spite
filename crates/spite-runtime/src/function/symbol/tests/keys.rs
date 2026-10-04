use super::*;

#[test]
fn property_access_keeps_symbol_identity_and_string_names_separate() {
    let mut realm = realm_with_symbols();
    check(&mut realm, "s[s]===undefined && (1n)[s]===undefined");
    check(
        &mut realm,
        "let o={name:1,[s]:2,[other]:3};o[s]===2 && o[same]===2 && o[other]===3 && o.name===1 && s in o",
    );
    check(
        &mut realm,
        "o[s]++;o[s]+=4;o[s]===7 && o[other]===3 && delete o[s] && !(same in o) && o.name===1",
    );
    check(
        &mut realm,
        "globalThis[globalKey]=42;globalThis[globalKey]===42 && delete globalThis[globalKey]",
    );
    check(
        &mut realm,
        "let a=[1];a[zeroKey]=2;a[lengthKey]={valueOf:()=>{throw 7;}};a.length=0;a[zeroKey]===2 && a.length===0 && typeof a[lengthKey]==='object'",
    );
    check(
        &mut realm,
        "let text=Object('a');text[zeroKey]='b';text[lengthKey]=10;text[0]==='a' && text.length===1 && text[zeroKey]==='b' && text[lengthKey]===10",
    );
    check(
        &mut realm,
        "function f(x){arguments[zeroKey]=4;arguments[lengthKey]=5;return x===1 && arguments[0]===1 && arguments.length===1 && arguments[zeroKey]===4 && arguments[lengthKey]===5;}f(1)",
    );
}

#[test]
fn computed_key_conversion_precedes_literal_values_and_is_cached_for_updates() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let log='',k={toString:()=>{log+='k';return s;}},o={[k]:(log+='v',1)};log==='kv' && o[s]===1",
    );
    check(
        &mut realm,
        "log='';o[k]=(log+='v',2);log==='vk' && o[s]===2",
    );
    check(
        &mut realm,
        "log='';o[k]+=(log+='v',3);log==='kv' && o[s]===5",
    );
    check(&mut realm, "log='';o[k]++;log==='k' && o[s]===6");
    check(&mut realm, "log='';delete o[k];log==='k' && !(s in o)");
    check(
        &mut realm,
        "let hinted={[convert]:(hint)=>{log+=hint;return s;}};log='';o[hinted]=9;log==='string' && o[s]===9",
    );
    check(&mut realm, "log='';hinted in o;log==='string'");
}

#[test]
fn descriptors_and_inherited_accessors_preserve_original_receivers() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let proto={},o=Object.create(proto),seen;Object.defineProperty(proto,s,{get:function(){return this;},set:function(v){seen=this;this.x=v;},configurable:true});o[s]===o && proto[s]===proto",
    );
    check(
        &mut realm,
        "o[s]=7;seen===o && o.x===7 && !Object.hasOwn(o,s) && !o.propertyIsEnumerable(s) && proto.propertyIsEnumerable(s)===false",
    );
    check(
        &mut realm,
        "Object.defineProperty(o,s,{value:3,writable:true,enumerable:true,configurable:true});let d=Object.getOwnPropertyDescriptor(o,s);d.value===3 && d.writable && d.enumerable && d.configurable && o.hasOwnProperty(same)",
    );
    check(
        &mut realm,
        "Object.freeze(o);o[s]=10;o[s]===3 && Object.isFrozen(o) && !delete o[s]",
    );
    assert!(matches!(
        realm.eval("'use strict';o[s]=10"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert!(matches!(
        realm.eval("Object.defineProperty(o,s,{value:4})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn enumerable_copy_orders_strings_before_symbols_and_observes_live_descriptors() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let log='',source={},target={};Object.defineProperty(source,s,{get:()=>{log+='s';return 4;},enumerable:true});Object.defineProperty(source,'x',{get:()=>{log+='x';return 3;},enumerable:true});Object.defineProperty(source,'2',{get:()=>{log+='2';return 2;},enumerable:true});Object.defineProperty(source,'1',{get:()=>{log+='1';return 1;},enumerable:true});Object.defineProperty(source,other,{get:()=>{throw 8;}});Object.assign(target,source);log==='12xs' && target[s]===4 && !Object.hasOwn(target,other)",
    );
    check(
        &mut realm,
        "let changing={x:1,[s]:2,[other]:3};Object.defineProperty(changing,'x',{get:()=>{delete changing[s];changing[empty]=4;return 5;}});let copied=Object.assign({},changing);copied.x===5 && !(s in copied) && copied[other]===3 && !(empty in copied)",
    );
    check(
        &mut realm,
        "let setters=Object.create({});let received;Object.defineProperty(Object.getPrototypeOf(setters),s,{set:function(v){received=this;this.x=v;}});Object.assign(setters,{[s]:8});received===setters && setters.x===8 && !Object.hasOwn(setters,s)",
    );
}

#[test]
fn descriptor_collections_and_integrity_operations_include_symbols() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let source={[s]:4,[other]:5};Object.defineProperty(source,empty,{value:6});let ds=Object.getOwnPropertyDescriptors(source);ds[s].value===4 && ds[other].enumerable && !ds[empty].enumerable && Object.hasOwn(ds,empty)",
    );
    check(
        &mut realm,
        "let made=Object.create(null,ds);made[s]===4 && made[other]===5 && made[empty]===6 && Object.getPrototypeOf(made)===null",
    );
    check(
        &mut realm,
        "Object.seal(made);Object.isSealed(made) && !Object.isFrozen(made) && !delete made[s]",
    );
    check(
        &mut realm,
        "Object.freeze(made);Object.isFrozen(made) && !Object.getOwnPropertyDescriptor(made,s).writable",
    );
    check(
        &mut realm,
        "let log='',props={x:{value:1}},out={};Object.defineProperty(props,s,{get:()=>{log+='s';throw 7;},enumerable:true});let caught=false;try{Object.defineProperties(out,props);}catch(e){caught=e===7;}caught && log==='s' && !Object.hasOwn(out,'x')",
    );
}

#[test]
fn inferred_names_distinguish_undefined_empty_and_utf16_descriptions() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let o={[s]:function(){},[unnamed]:()=>0,[empty]:()=>0,[raw]:()=>0,[other]:function explicit(){}};o[s].name==='[name]' && o[unnamed].name==='' && o[empty].name==='[]' && o[raw].name==='[\\uD800\\u0000\\uDC00]' && o[other].name==='explicit'",
    );
    check(
        &mut realm,
        "let d=Object.getOwnPropertyDescriptor(o[s],'name');!d.writable && !d.enumerable && d.configurable",
    );
    realm.limits.max_string_units = 5;
    assert!(matches!(
        realm.eval("({[s]:()=>0})"),
        Err(Error::Limit { .. })
    ));
    // Symbols do not require description formatting for non-anonymous values.
    assert!(realm.eval("({[s]:1})").is_ok());
}

#[test]
fn host_symbol_lookup_is_constant_work_and_keeps_getter_semantics() {
    let mut realm = realm_with_symbols();
    let object = realm.eval("let o={[s]:7};o").unwrap();
    let Value::Symbol(symbol) = realm.eval("s").unwrap() else {
        panic!("symbol");
    };
    assert_eq!(
        realm.read_property(&object, &symbol),
        Ok(Value::Number(7.0))
    );
    assert_eq!(
        realm.read_property(&object, &spite_core::PropertyKey::Symbol(symbol.clone())),
        Ok(Value::Number(7.0))
    );
    realm
        .eval("Object.defineProperty(o,s,{get:function(){return this;}})")
        .unwrap();
    assert_eq!(realm.read_property(&object, &symbol), Ok(object.clone()));
    let huge = JsSymbol::new(Some(JsString::from_code_units(vec![0x61; 100_000])));
    realm.limits.max_steps = 30;
    realm.limits.max_string_units = 1;
    assert_eq!(realm.read_property(&object, &huge), Ok(Value::Undefined));
}
