use super::*;
use crate::{function::Callable, object::Property};

fn object(value: Value) -> ObjectHandle {
    let Value::Object(handle) = value else {
        panic!("object expected");
    };
    handle
}

#[test]
fn detached_methods_retain_home_objects_and_unrooted_cycles_are_collected() {
    let mut realm = Realm::default();
    let method = object(
        realm
            .eval("let method=({payload:{n:7},m(){return this.payload;}}).m;method")
            .unwrap(),
    );
    let Some(Callable::Method(code)) = realm.objects.inspect(&method).unwrap().callable() else {
        panic!("method expected");
    };
    let home = code.home_object.clone();
    let payload = object(
        realm
            .objects
            .inspect(&home)
            .unwrap()
            .own_property(&JsString::from("payload"))
            .unwrap()
            .as_data()
            .unwrap()
            .value
            .clone(),
    );
    // The hidden home edge survives changes to the method's public prototype.
    realm.eval("Object.setPrototypeOf(method,null)").unwrap();
    realm.collect(30_000).unwrap();
    assert!(realm.objects.inspect(&home).is_ok());
    assert!(realm.objects.inspect(&payload).is_ok());
    assert_eq!(
        realm.call(
            Value::Object(method.clone()),
            Value::Object(home.clone()),
            vec![],
            Span::new(0, 0)
        ),
        Ok(Value::Object(payload.clone())),
    );
    realm.eval("method=null").unwrap();
    realm.collect(30_000).unwrap();
    for handle in [&method, &home, &payload] {
        assert!(matches!(
            realm.objects.inspect(handle),
            Err(crate::object::Error::Heap(spite_heap::Error::StaleHandle))
        ));
    }
}

#[test]
fn method_creation_rejects_foreign_and_stale_home_objects_before_allocation() {
    let mut realm = Realm::default();
    let handle = object(realm.eval("let m=({m(){}}).m;m").unwrap());
    let Some(Callable::Method(method)) = realm.objects.inspect(&handle).unwrap().callable() else {
        panic!("method expected");
    };
    let method = method.as_ref().clone();
    let prototype = realm
        .intrinsics
        .as_ref()
        .unwrap()
        .function_prototype
        .clone();
    let mut foreign = Realm::default();
    let foreign_home = object(foreign.eval("({})").unwrap());
    let stale_home = object(realm.eval("({})").unwrap());
    let before = realm.collect(30_000).unwrap().live;
    for (home, expected) in [
        (foreign_home, spite_heap::Error::ForeignHandle),
        (stale_home, spite_heap::Error::StaleHandle),
    ] {
        let result = realm.objects.create_method(
            &prototype,
            MethodFunction {
                code: method.code.clone(),
                home_object: home,
            },
        );
        assert_eq!(result, Err(crate::object::Error::Heap(expected)));
        let collection = realm.collect(30_000).unwrap();
        assert_eq!(collection.live, before);
        assert_eq!(collection.reclaimed, 0);
    }
}

#[test]
fn accessor_name_prefixes_obey_utf16_limits_and_host_failures_bypass_handlers() {
    let mut realm = Realm::default();
    realm.eval("let key=Symbol('xyz'),flag=0").unwrap();
    realm.limits.max_string_units = 8;
    assert!(matches!(
        realm.eval("try{({get [key](){}});}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. }),
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    realm.limits.max_string_units = 9;
    let home = object(realm.eval("({get [key](){}})").unwrap());
    let Value::Symbol(key) = realm.eval("key").unwrap() else {
        panic!("symbol expected");
    };
    let Property::Accessor(accessor) = realm
        .objects
        .inspect(&home)
        .unwrap()
        .own_property(&key)
        .unwrap()
    else {
        panic!("accessor expected");
    };
    let getter = realm
        .objects
        .inspect(accessor.get.as_ref().unwrap())
        .unwrap();
    assert_eq!(
        getter
            .own_property(&JsString::from("name"))
            .unwrap()
            .as_data()
            .unwrap()
            .value,
        Value::String(JsString::from("get [xyz]"))
    );
}
