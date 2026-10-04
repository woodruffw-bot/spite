//! Reflect set: exact receivers, checked descriptors, and exotic definitions.

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
fn target_validation_and_key_conversion_precede_receiver_descriptor_reads() {
    for target in ["undefined", "null", "true", "1", "'s'", "1n", "Symbol()"] {
        check(&format!(
            "let reads=0,caught=false;try{{Reflect.set({target},{{toString(){{reads++;throw 7;}}}},1,{{}});}}catch(e){{caught=e instanceof TypeError;}}caught && reads===0"
        ));
    }
    check(
        "let calls=0,s=Symbol(),o={},key={[Symbol.toPrimitive](hint){if(hint!=='string')throw 7;calls++;return s;}};Reflect.set(o,key,7)===true && o[s]===7 && calls===1",
    );
    check(
        "let caught=false,o={};try{Reflect.set(o,{toString(){throw 7;}},1);}catch(e){caught=e===7;}caught && Object.keys(o).length===0",
    );
    check(
        "let receiver={x:1},key={toString(){Object.defineProperty(receiver,'x',{writable:false});return 'x';}};Reflect.set({},key,2,receiver)===false && receiver.x===1",
    );
}

#[test]
fn writable_data_targets_define_on_receiver_and_bypass_receiver_prototypes() {
    check(
        "let target={x:1},receiver={};Reflect.set(target,'x',2,receiver)===true && target.x===1 && receiver.x===2 && Object.getOwnPropertyDescriptor(receiver,'x').writable && Object.getOwnPropertyDescriptor(receiver,'x').enumerable && Object.getOwnPropertyDescriptor(receiver,'x').configurable",
    );
    check(
        "let calls=0,receiver=Object.create({set x(v){calls++;throw 7;}});Reflect.set({},'x',2,receiver)===true && receiver.x===2 && calls===0 && Object.hasOwn(receiver,'x')",
    );
    check(
        "let p={};Object.defineProperty(p,'x',{value:1});let receiver=Object.create(p);Reflect.set({x:0},'x',2,receiver)===true && receiver.x===2 && p.x===1",
    );
    check(
        "let target={};Object.preventExtensions(target);let receiver={};Reflect.set(target,'x',2,receiver)===true && receiver.x===2 && !Object.hasOwn(target,'x')",
    );
    check(
        "let o={x:1};Reflect.set(o,'x',2)===true && o.x===2 && Reflect.set(o,'new',3)===true && o.new===3",
    );
}

#[test]
fn data_and_missing_setter_rejections_are_boolean_even_in_strict_callers() {
    check(
        "'use strict';let target={},receiver={x:2};Object.defineProperty(target,'x',{value:1});Reflect.set(target,'x',3,receiver)===false && target.x===1 && receiver.x===2",
    );
    check(
        "let target={x:1},receiver={};Object.defineProperty(receiver,'x',{value:2});Reflect.set(target,'x',3,receiver)===false && receiver.x===2 && target.x===1",
    );
    check(
        "let calls=0,receiver={set x(v){calls++;}};Reflect.set({},'x',3,receiver)===false && calls===0 && Object.getOwnPropertyDescriptor(receiver,'x').set!==undefined",
    );
    check(
        "let reads=0,target={get x(){reads++;throw 7;}},receiver={};Reflect.set(target,'x',2,receiver)===false && reads===0 && !Object.hasOwn(receiver,'x')",
    );
    check(
        "let receiver={};Object.preventExtensions(receiver);Reflect.set({},'x',2,receiver)===false && !Object.hasOwn(receiver,'x')",
    );
    for receiver in ["undefined", "null", "true", "1", "'s'", "1n", "Symbol()"] {
        check(&format!(
            "let target={{x:1}};Reflect.set(target,'x',2,{receiver})===false && Reflect.set(target,'missing',2,{receiver})===false && target.x===1 && !Object.hasOwn(target,'missing')"
        ));
    }
}

#[test]
fn setters_receive_exact_receivers_and_return_true_independently_of_their_result() {
    check(
        "let calls=0;Reflect.set({set normalize(v){calls++;}},'normalize',1,String.prototype)===true && calls===1",
    );
    check(
        "let calls=0,seen,got,p={set x(v){'use strict';if(arguments.length!==1)throw 7;calls++;seen=this;got=v;return false;}},target=Object.create(p),receiver={};Reflect.set(target,'x',7,receiver)===true && calls===1 && seen===receiver && got===7 && !Object.hasOwn(target,'x') && !Object.hasOwn(receiver,'x')",
    );
    check(
        "let seen,target={set x(v){'use strict';seen=this;}};Reflect.set(target,'x',7,undefined)===true && seen===undefined && Reflect.set(target,'x',7,null)===true && seen===null && Reflect.set(target,'x',7,3)===true && seen===3",
    );
    check(
        "let seen,target={set x(v){seen=this;}};Reflect.set(target,'x',7,null)===true && seen===globalThis && Reflect.set(target,'x',7,3)===true && seen.valueOf()===3",
    );
    check("let target={set x(v){return 3;}};Reflect.set(target,'x',1,String.prototype)===true");
    check(
        "let caught=false,target={set x(v){throw 7;}};try{Reflect.set(target,'x',1,{});}catch(e){caught=e===7;}caught",
    );
}

#[test]
fn array_string_and_argument_receivers_use_their_internal_definitions() {
    check(
        "let a=[1,2,3];Reflect.set({},'1',7,a)===true && a[1]===7 && Reflect.set({},'4',8,a)===true && a.length===5 && a[4]===8",
    );
    check(
        "let calls=0,a=[1,2,3],length={valueOf(){calls++;return 2;}};Reflect.set({},'length',length,a)===true && calls===2 && a.length===2",
    );
    check(
        "let a=[1,2,3];Object.defineProperty(a,'1',{configurable:false});Reflect.set({},'length',0,a)===false && a.length===2 && a[1]===2 && !Object.hasOwn(a,2)",
    );
    check(
        "let calls=0,a=[];Object.defineProperty(a,'length',{writable:false});Reflect.set({},'length',{valueOf(){calls++;return 1;}},a)===false && Reflect.set({},0,1,a)===false && calls===0 && a.length===0",
    );
    check(
        "let caught=false,a=[];try{Reflect.set({},'length',1.5,a);}catch(e){caught=e instanceof RangeError;}caught && a.length===0",
    );
    check(
        "let s=Object('ab');Reflect.set({},0,'x',s)===false && Reflect.set({},'length',3,s)===false && Reflect.set({},'x',7,s)===true && s[0]==='a' && s.x===7",
    );
    check(
        "function f(a){return Reflect.set({},0,7,arguments)===true && a===7 && arguments[0]===7;}f(1)",
    );
}

#[test]
fn ordinary_assignments_share_checked_setter_and_exotic_paths() {
    check(
        "let receiver={y:1},p={set x(v){this.y=v;}},o=Object.create(p);o.x=7;o.y===7 && !Object.hasOwn(o,'x')",
    );
    check(
        "let calls=0;Object.defineProperty(Number.prototype,'x',{set(v){'use strict';if(this!==3 || v!==7)throw 8;calls++;},configurable:true});(3).x=7;calls===1",
    );
    check("let a=[1,2,3];a.length=2;a.length===2 && !Object.hasOwn(a,2)");
    check(
        "'use strict';let o={};Object.defineProperty(o,'x',{value:1});let caught=false;try{o.x=2;}catch(e){caught=e instanceof TypeError;}caught && Reflect.set(o,'x',2)===false && o.x===1",
    );
}

#[test]
fn complete_reflect_own_keys_and_integrity_operations_are_exposed() {
    check(
        "let names='apply,construct,defineProperty,deleteProperty,get,getOwnPropertyDescriptor,getPrototypeOf,has,isExtensible,ownKeys,preventExtensions,set,setPrototypeOf',keys=Reflect.ownKeys(Reflect);keys.length===14 && keys.slice(0,13).join(',')===names && keys[13]===Symbol.toStringTag && Object.getOwnPropertyNames(Reflect).join(',')===names && Object.keys(Reflect).length===0 && Object.getOwnPropertySymbols(Reflect)[0]===Symbol.toStringTag",
    );
    check(
        "Object.freeze(Reflect);Object.isFrozen(Reflect) && Reflect.set(Reflect,'set',7)===false && Reflect.apply((x)=>x+1,null,[2])===3 && Object.keys(Object.assign({},Reflect)).length===0",
    );
    check(
        "delete Reflect.set;Reflect.has(Reflect,'set')===false && Reflect.get(Reflect,'set')===undefined && !Object.hasOwn(Reflect,'set') && Reflect.ownKeys(Reflect).length===13",
    );
    check("let count=0;for(let key in Reflect){count++;}count===0");
}

#[test]
fn metadata_and_saved_intrinsics_survive_deletion_and_collection() {
    check(
        "let d=Object.getOwnPropertyDescriptor(Reflect,'set'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='set' && !n.writable && !n.enumerable && n.configurable && l.value===3 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')",
    );
    check("let caught=false;try{new Reflect.set();}catch(e){caught=e instanceof TypeError;}caught");
    let mut realm = Realm::default();
    realm
        .eval("let set=Reflect.set;delete Reflect.set;delete globalThis.Reflect")
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("let o={};set(o,'x',7) && o.x===7"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn missing_intrinsic_descriptors_and_unsupported_setters_skip_pending_handlers() {
    for operation in [
        "Reflect.set(String.prototype,'normalize',1)",
        "Reflect.set({},'normalize',1,String.prototype)",
        "Reflect.set({set x(v){Math;}},'x',1)",
        "Reflect.set({},'length',{valueOf(){Math;}},[])",
        "String.prototype.normalize=1",
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
