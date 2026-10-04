//! Complete edition-17 shared Iterator prototype inventory (27.1.3.3).

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn own_strings_and_symbols_follow_the_complete_specification_inventory() {
    check(
        "let p=Iterator.prototype,names=Object.getOwnPropertyNames(p),symbols=Object.getOwnPropertySymbols(p),keys=Reflect.ownKeys(p);names.join(',')==='constructor,drop,every,filter,find,flatMap,forEach,map,reduce,some,take,toArray' && symbols.length===2 && symbols[0]===Symbol.toStringTag && symbols[1]===Symbol.iterator && keys.length===14 && keys.slice(0,12).join(',')===names.join(',') && keys[12]===symbols[0] && keys[13]===symbols[1] && Object.keys(p).length===0 && Object.values(p).length===0 && Object.entries(p).length===0",
    );
    check(
        "let p=Iterator.prototype;!Object.hasOwn(p,'dispose') && !Object.hasOwn(p,'zip') && !Object.hasOwn(p,'chunks') && !Object.hasOwn(p,'includes') && !Object.hasOwn(p,'join') && !Object.hasOwn(p,'windows') && !Object.hasOwn(p,'next') && !Object.hasOwn(p,'return')",
    );
}

#[test]
fn every_helper_has_standard_function_and_property_attributes() {
    for (name, length) in [
        ("drop", 1),
        ("every", 1),
        ("filter", 1),
        ("find", 1),
        ("flatMap", 1),
        ("forEach", 1),
        ("map", 1),
        ("reduce", 1),
        ("some", 1),
        ("take", 1),
        ("toArray", 0),
    ] {
        check(&format!(
            "let p=Iterator.prototype,d=Object.getOwnPropertyDescriptor(p,'{name}'),f=d.value;d.writable && !d.enumerable && d.configurable && f.name==='{name}' && f.length==={length} && Object.getPrototypeOf(f)===Function.prototype && !Object.hasOwn(f,'prototype') && !Object.hasOwn(d,'get') && !Object.hasOwn(d,'set')"
        ));
    }
    check(
        "let d=Object.getOwnPropertyDescriptor(Iterator.prototype,Symbol.iterator),f=d.value;d.writable && !d.enumerable && d.configurable && f.name==='[Symbol.iterator]' && f.length===0 && !Object.hasOwn(f,'prototype') && f.call(null)===null && f.call(undefined)===undefined",
    );
}

#[test]
fn descriptor_copying_preserves_accessors_without_observing_them() {
    check(
        "let p=Iterator.prototype,d=Object.getOwnPropertyDescriptors(p),q=Object.create(null,d),c=Object.getOwnPropertyDescriptor(p,'constructor'),t=Object.getOwnPropertyDescriptor(p,Symbol.toStringTag);Reflect.ownKeys(d).length===14 && Reflect.ownKeys(q).length===14 && d.constructor.get===c.get && d.constructor.set===c.set && d[Symbol.toStringTag].get===t.get && d[Symbol.toStringTag].set===t.set && q.constructor===Iterator && q[Symbol.toStringTag]==='Iterator' && q.map===p.map && q[Symbol.iterator]()===q",
    );
    check(
        "let p=Iterator.prototype,c=Object.getOwnPropertyDescriptor(p,'constructor'),t=Object.getOwnPropertyDescriptor(p,Symbol.toStringTag),calls=0;Object.defineProperty(p,'constructor',{get(){calls++;throw 7;}});Object.defineProperty(p,Symbol.toStringTag,{get(){calls++;throw 8;}});let d=Object.getOwnPropertyDescriptors(p),copy=Object.assign({},p);Reflect.ownKeys(d).length===14 && Reflect.ownKeys(copy).length===0 && Object.keys(p).length===0 && calls===0 && d.constructor.set===c.set && d[Symbol.toStringTag].set===t.set",
    );
}

#[test]
fn sealing_preserves_method_writability_and_accessor_identity() {
    check(
        "let p=Iterator.prototype,c=Object.getOwnPropertyDescriptor(p,'constructor'),t=Object.getOwnPropertyDescriptor(p,Symbol.toStringTag),f=p.map;Object.seal(p);let d=Object.getOwnPropertyDescriptors(p);Object.isSealed(p) && !Object.isFrozen(p) && !Object.isExtensible(p) && d.map.writable && !d.map.configurable && d.constructor.get===c.get && d.constructor.set===c.set && !d.constructor.configurable && d[Symbol.toStringTag].get===t.get && d[Symbol.toStringTag].set===t.set && !d[Symbol.toStringTag].configurable && !Reflect.deleteProperty(p,'map') && Reflect.set(p,'map',7) && p.map===7 && !Reflect.defineProperty(p,'newProperty',{value:8}) && f.call([7].values(),x=>x).next().value===7",
    );
}

#[test]
fn freezing_preserves_protected_setters_and_allows_inheriting_own_overrides() {
    check(
        "let p=Iterator.prototype,c=Object.getOwnPropertyDescriptor(p,'constructor'),t=Object.getOwnPropertyDescriptor(p,Symbol.toStringTag),calls=0;Object.defineProperty(p,'constructor',{get(){calls++;throw 7;}});Object.defineProperty(p,Symbol.toStringTag,{get(){calls++;throw 8;}});Object.freeze(p);let d=Object.getOwnPropertyDescriptors(p);Object.isFrozen(p) && Object.isSealed(p) && !d.map.writable && !d.map.configurable && !d[Symbol.iterator].writable && !d[Symbol.iterator].configurable && d.constructor.set===c.set && d[Symbol.toStringTag].set===t.set && calls===0 && !Reflect.set(p,'map',7) && !Reflect.deleteProperty(p,Symbol.iterator)",
    );
    check(
        "let p=Iterator.prototype,c=Object.getOwnPropertyDescriptor(p,'constructor'),t=Object.getOwnPropertyDescriptor(p,Symbol.toStringTag),q=Object.create(p),sentinel={},count=0;Object.freeze(p);q.constructor=sentinel;q[Symbol.toStringTag]='Mine';try{c.set.call(p,7);}catch(e){if(e instanceof TypeError)count++;}try{t.set.call(p,'x');}catch(e){if(e instanceof TypeError)count++;}let cd=Object.getOwnPropertyDescriptor(q,'constructor'),td=Object.getOwnPropertyDescriptor(q,Symbol.toStringTag);count===2 && p.constructor===Iterator && p[Symbol.toStringTag]==='Iterator' && q.constructor===sentinel && q[Symbol.toStringTag]==='Mine' && cd.writable && cd.enumerable && cd.configurable && td.writable && td.enumerable && td.configurable && [7].values().map(x=>x).next().value===7 && Object.isFrozen(p)",
    );
}

#[test]
fn enumeration_of_inheriting_iterators_reaches_complete_prototypes() {
    check(
        "let keys=[],h=[7].values().flatMap(x=>[x]);h.x=1;Iterator.prototype.extra=2;for(let key in h)keys.push(key);keys.join(',')==='x,extra' && h.toArray().join(',')==='7'",
    );
    check("let keys=[];for(let key in 'a'[Symbol.iterator]())keys.push(key);keys.length===0");
    check(
        "let keys=[];for(let key in Iterator.from({next:()=>({done:true})}))keys.push(key);keys.length===0",
    );
}

#[test]
fn delete_and_redefine_use_ordinary_creation_order_and_intrinsics_remain_rooted() {
    check(
        "let p=Iterator.prototype,f=p.map;delete p.map;let absent=!Object.hasOwn(p,'map') && p.map===undefined;Object.defineProperty(p,'map',{value:f,writable:true,configurable:true});let keys=Reflect.ownKeys(p);absent && keys.length===14 && keys[11]==='map' && keys[12]===Symbol.toStringTag && keys[13]===Symbol.iterator && p.map.call([7].values(),x=>x).next().value===7",
    );
    let mut realm = Realm::default();
    realm.eval("let I=Iterator,p=I.prototype;for(let key of Reflect.ownKeys(p))delete p[key];globalThis.Iterator=null;").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(realm.eval("Reflect.ownKeys(p).length===0 && Object.getPrototypeOf(Object.getPrototypeOf([7].values()))===p && Object.getPrototypeOf(Object.getPrototypeOf('a'[Symbol.iterator]()))===p && Object.getPrototypeOf(Object.getPrototypeOf(I.from({next:()=>({done:true})})))===p"),Ok(Value::Boolean(true)));
}
