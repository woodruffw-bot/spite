use super::*;

#[test]
fn species_getter_is_generic_and_preserves_primitive_receivers() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor,key=S.species,d=Object.getOwnPropertyDescriptor(Array,key),get=d.get;Array[key]===Array && d.set===undefined && !d.enumerable && d.configurable && get.length===0 && get.name==='get [Symbol.species]'",
    );
    check(
        &mut realm,
        "get.call(undefined)===undefined && get.call(null)===null && get.call(s)===s && Object.is(get.call(-0),-0) && get.call(1n)===1n && get.call('x')==='x'",
    );
    check(
        &mut realm,
        "function Child(){}Object.setPrototypeOf(Child,Array);Child[key]===Child",
    );
    check(&mut realm, "Array[key]=1;Array[key]===Array");
    assert!(matches!(
        realm.eval("'use strict';Array[key]=1"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert!(matches!(
        realm.eval("new get"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn unscopables_has_exact_null_prototype_data_and_standard_attributes() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor,key=S.unscopables,d=Object.getOwnPropertyDescriptor(Array.prototype,key),u=d.value;Object.getPrototypeOf(u)===null && !d.writable && !d.enumerable && d.configurable && Array.prototype[key]===u && [][key]===u",
    );
    let expected = [
        "at",
        "copyWithin",
        "entries",
        "fill",
        "find",
        "findIndex",
        "findLast",
        "findLastIndex",
        "flat",
        "flatMap",
        "includes",
        "keys",
        "toReversed",
        "toSorted",
        "toSpliced",
        "values",
    ];
    for name in expected {
        check(
            &mut realm,
            &format!(
                "d=Object.getOwnPropertyDescriptor(u,'{name}');d.value===true && d.writable && d.enumerable && d.configurable"
            ),
        );
    }
    let Value::Object(table) = realm.eval("u").unwrap() else {
        panic!("table");
    };
    let keys = realm.own_property_keys(&table, Span::new(0, 0)).unwrap();
    assert_eq!(
        keys,
        expected
            .into_iter()
            .map(spite_core::PropertyKey::from)
            .collect::<Vec<_>>()
    );
    check(
        &mut realm,
        "!Object.hasOwn(u,'with') && !Object.hasOwn(u,'push') && !Object.hasOwn(u,'__proto__')",
    );
    check(
        &mut realm,
        "u.at=false;u.at===false && Array.prototype[key]===u",
    );
    check(
        &mut realm,
        "Array.prototype[key]=null;Array.prototype[key]===u",
    );
    assert!(matches!(
        realm.eval("'use strict';Array.prototype[key]=null"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn change_by_copy_algorithms_and_array_of_do_not_observe_species() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor,calls=0;Object.defineProperty(Array,S.species,{get:()=>{calls++;throw 7;}});let a=[3,1,2];a.toReversed().join()==='2,1,3' && a.toSorted().join()==='1,2,3' && a.with(1,4).join()==='3,4,2' && a.toSpliced(1,1,4).join()==='3,4,2' && Array.of(1,2).join()==='1,2' && calls===0",
    );
}

#[test]
fn symbol_properties_are_configurable_and_saved_intrinsics_survive_collection() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor,get=Object.getOwnPropertyDescriptor(Array,S.species).get,u=Array.prototype[S.unscopables];delete Array[S.species] && delete Array.prototype[S.unscopables] && !(S.species in Array) && !(S.unscopables in Array.prototype)",
    );
    realm.collect(30_000).unwrap();
    check(
        &mut realm,
        "get.call(Array)===Array && u.find===true && Object.getPrototypeOf(u)===null",
    );
    check(
        &mut realm,
        "Object.defineProperty(Array,S.species,{value:s});Array[S.species]===s",
    );
    check(
        &mut realm,
        "Object.defineProperty(Array.prototype,S.unscopables,{value:null});[][S.unscopables]===null",
    );
}
