use super::*;

#[test]
fn iterator_aliases_and_prototype_descriptors_are_standard() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor,key=S.iterator,d=Object.getOwnPropertyDescriptor(Array.prototype,key),i=[1].values(),p=Object.getPrototypeOf(i),base=Object.getPrototypeOf(p);d.value===Array.prototype.values && d.value.name==='values' && d.writable && !d.enumerable && d.configurable && i[key]()===i && Object.getPrototypeOf(base)===Object.prototype",
    );
    check(
        &mut realm,
        "d=Object.getOwnPropertyDescriptor(p,'next');d.value===i.next && d.value.name==='next' && d.value.length===0 && d.writable && !d.enumerable && d.configurable",
    );
    check(
        &mut realm,
        "d=Object.getOwnPropertyDescriptor(p,S.toStringTag);d.value==='Array Iterator' && !d.writable && !d.enumerable && d.configurable && Object.prototype.toString.call(i)==='[object Array Iterator]'",
    );
    check(
        &mut realm,
        "let identity=base[key];identity.name==='[Symbol.iterator]' && identity.length===0 && identity.call(null)===null && identity.call(s)===s && identity.call(undefined)===undefined",
    );
    for source in [
        "base.constructor",
        "base.map",
        "Object.getOwnPropertyDescriptors(base)",
    ] {
        assert!(
            matches!(realm.eval(source), Err(Error::Unsupported { .. })),
            "{source}"
        );
    }
}

#[test]
fn mapped_and_unmapped_arguments_use_the_intrinsic_values_callable() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor,key=S.iterator,values=Array.prototype.values;Array.prototype.values=()=>{throw 7;};function f(x){let i=arguments[key]();x=2;let d=Object.getOwnPropertyDescriptor(arguments,key);return i.next().value===2 && i.next().done && d.value===values && d.writable && !d.enumerable && d.configurable;}f(1)",
    );
    check(
        &mut realm,
        "function strict(x){'use strict';let i=arguments[key]();x=2;return i.next().value===1 && i.next().done && arguments[key]===values;}strict(1)",
    );
    check(
        &mut realm,
        "function defaults(x=2){return arguments[key]===values && arguments[key]().next().done;}defaults()",
    );
    check(
        &mut realm,
        "[1][key]().next().value===1 && Array.prototype[key]===values",
    );
}

#[test]
fn iterators_trace_unreferenced_sources_and_release_them_at_completion() {
    let mut realm = realm_with_symbols();
    let Value::Object(iterator) = realm.eval("let i=[{x:7}].values();i").unwrap() else {
        panic!("iterator");
    };
    let source = realm
        .objects
        .inspect(&iterator)
        .unwrap()
        .array_iterator()
        .unwrap()
        .array
        .as_ref()
        .unwrap()
        .clone();
    realm.collect(usize::MAX).unwrap();
    check(&mut realm, "i.next().value.x===7 && i.next().done");
    realm.collect(usize::MAX).unwrap();
    assert!(matches!(
        realm.objects.inspect(&source),
        Err(crate::object::Error::Heap(spite_heap::Error::StaleHandle))
    ));
    check(&mut realm, "i.next().done && i.next().value===undefined");
}

#[test]
fn iterator_indices_cover_the_entire_array_like_length_range() {
    let mut realm = realm_with_symbols();
    let Value::Object(iterator) = realm
        .eval("let i=Array.prototype.keys.call({length:Infinity});i")
        .unwrap()
    else {
        panic!("iterator");
    };
    realm
        .objects
        .set_array_iterator_index(&iterator, 9_007_199_254_740_990)
        .unwrap();
    check(
        &mut realm,
        "i.next().value===9007199254740990 && i.next().done",
    );
}

#[test]
fn entries_ignore_species() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor;Object.defineProperty(Array,S.species,{get:()=>{throw 7;}});let i=[8].entries();i.next().value[1]===8",
    );
}

#[test]
fn recursive_next_length_getters_obey_the_host_limit_and_restore_call_state() {
    std::thread::Builder::new().stack_size(2*1024*1024).spawn(|| {
        let mut realm=realm_with_symbols();
        realm.eval("let o={0:7},i=Array.prototype.values.call(o),flag=0;Object.defineProperty(o,'length',{get:()=>i.next(),configurable:true})").unwrap();
        assert!(matches!(realm.eval("try{i.next();}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})));
        check(&mut realm,"flag===0 && (Object.defineProperty(o,'length',{value:1}),i.next().value===7) && i.next().done");
    }).unwrap().join().unwrap();
}
