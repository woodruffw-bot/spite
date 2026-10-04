//! Math.random range, realm separation, metadata, and complete Math reflection.

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
fn random_values_have_positive_sign_and_remain_in_the_half_open_unit_interval() {
    check(
        "let good=true;for(let i=0;i<256;i++){let x=Math.random();if(typeof x!=='number' || x<0 || x>=1 || !Number.isFinite(x) || Object.is(x,-0))good=false;}good",
    );
}

#[test]
fn distinct_realms_keep_separate_sequences_and_ignore_receiver_and_arguments() {
    let mut first = Realm::default();
    let mut second = Realm::default();
    let first_values: Vec<_> = (0..16)
        .map(|_| first.eval("Math.random()").unwrap())
        .collect();
    let second_values: Vec<_> = (0..16)
        .map(|_| second.eval("Math.random()").unwrap())
        .collect();
    assert_ne!(first_values, second_values);
    check(
        "let x=Math.random.call(Symbol(),{[Symbol.toPrimitive](){throw 7;},valueOf(){throw 8;}},1n);x>=0 && x<1",
    );
}

#[test]
fn random_has_standard_metadata_and_is_not_a_constructor() {
    check(
        "let d=Object.getOwnPropertyDescriptor(Math,'random'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='random' && !n.writable && !n.enumerable && n.configurable && l.value===0 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')",
    );
    check("let caught=false;try{new Math.random();}catch(e){caught=e instanceof TypeError;}caught");
}

#[test]
fn complete_math_own_reflection_and_integrity_operations_preserve_standard_attributes() {
    check(
        "let keys=Reflect.ownKeys(Math),names=Object.getOwnPropertyNames(Math),symbols=Object.getOwnPropertySymbols(Math),descriptors=Object.getOwnPropertyDescriptors(Math);keys.length===46 && names.length===45 && symbols.length===1 && symbols[0]===Symbol.toStringTag && keys[45]===Symbol.toStringTag && descriptors.PI.value===Math.PI && descriptors.sumPrecise.value===Math.sumPrecise && Object.keys(Math).length===0 && Object.keys(Object.assign({},Math)).length===0",
    );
    check(
        "let count=0;for(let key in Math)count++;count===0 && Math.missing===undefined && Reflect.has(Math,'missing')===false && Object.getOwnPropertyDescriptor(Math,'missing')===undefined",
    );
    check(
        "Object.freeze(Math)===Math && Object.isFrozen(Math) && Object.isSealed(Math) && !Object.isExtensible(Math) && Reflect.set(Math,'random',7)===false && Reflect.deleteProperty(Math,'random')===false && typeof Math.random==='function' && Math.random()>=0",
    );
    check(
        "let calls=0;Object.defineProperty(Math,'custom',{get(){calls++;return 7;},enumerable:true,configurable:true});Object.freeze(Math);Object.isFrozen(Math) && Object.getOwnPropertyDescriptor(Math,'custom').get!==undefined && calls===0",
    );
}

#[test]
fn intrinsic_roots_retain_random_state_after_public_deletion_and_collection() {
    let mut realm = Realm::default();
    realm
        .eval("let random=Math.random;delete Math.random;delete globalThis.Math;")
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval(
            "let x=random(),y=random();x>=0 && x<1 && y>=0 && y<1 && typeof Math==='undefined'"
        ),
        Ok(Value::Boolean(true))
    );
}
