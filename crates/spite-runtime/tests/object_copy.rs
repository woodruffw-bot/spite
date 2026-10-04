//! Observable value copying and descriptor-preserving Object reflection.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn assign_boxes_targets_skips_nullish_sources_and_copies_only_enumerable_own_values() {
    check(
        "let o={};Object.assign(o)===o && Object.assign(o,null,undefined,true,1)===o && Object.assign(o,{x:1},{x:2,y:3})===o && o.x===2 && o.y===3",
    );
    check(
        "let n=Object.assign(3,{x:4}),b=Object.assign(false,{x:5});n instanceof Number && n.valueOf()===3 && n.x===4 && b instanceof Boolean && !b.valueOf() && b.x===5",
    );
    check(
        "let p={inherited:1},source=Object.create(p,{hidden:{value:2},x:{value:3,enumerable:true}});let o=Object.assign({},source);!Object.hasOwn(o,'hidden') && !Object.hasOwn(o,'inherited') && o.x===3 && Object.getOwnPropertyDescriptor(o,'x').writable && Object.getOwnPropertyDescriptor(o,'x').configurable",
    );
    for source in [
        "Object.assign()",
        "Object.assign(null,{})",
        "Object.assign(undefined)",
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
}

#[test]
fn assign_orders_getters_and_setters_and_rechecks_each_snapshotted_key() {
    check(
        "let log='',p={},s={};Object.defineProperty(p,'x',{set:function(v){if(this!==o)throw 1;log+='s'+v;}});Object.defineProperty(s,'x',{get:function(){if(this!==s)throw 2;log+='g';return 7;},enumerable:true});let o=Object.create(p);Object.assign(o,s)===o && log==='gs7' && !Object.hasOwn(o,'x')",
    );
    check(
        "let s={};Object.defineProperty(s,'a',{get:()=>{delete s.b;s.c=3;return 1;},enumerable:true});s.b=2;let o=Object.assign({},s);o.a===1 && !Object.hasOwn(o,'b') && !Object.hasOwn(o,'c')",
    );
    check(
        "let s={};Object.defineProperty(s,'a',{get:()=>{Object.defineProperty(s,'b',{enumerable:false});return 1;},enumerable:true});s.b=2;let o=Object.assign({},s);o.a===1 && !Object.hasOwn(o,'b')",
    );
    check(
        "let later={},first={};Object.defineProperty(first,'a',{get:()=>{later.x=7;return 1;},enumerable:true});let o=Object.assign({},first,later);o.a===1 && o.x===7",
    );
    let mut realm = Realm::default();
    let Value::Object(object) = realm.eval("Object.assign({},{b:1,10:2,2:3,a:4})").unwrap() else {
        panic!("target");
    };
    assert_eq!(
        realm.inspect_object(&object).unwrap().own_keys(),
        ["2", "10", "b", "a"].map(spite_core::PropertyKey::from)
    );
}

#[test]
fn assignment_abrupt_completions_preserve_earlier_copies_and_stop_later_sources() {
    let mut realm = Realm::default();
    realm.eval("let o={},read=false;Object.defineProperty(o,'b',{value:0});let s={a:1,b:2};Object.defineProperty(s,'c',{get:()=>{read=true;return 3;},enumerable:true})").unwrap();
    assert!(matches!(
        realm.eval("Object.assign(o,s,{later:1})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(
        realm.eval("o.a===1 && o.b===0 && !read && !Object.hasOwn(o,'later')"),
        Ok(Value::Boolean(true))
    );
    realm
        .eval(
            "let o2={},s2={a:1};Object.defineProperty(s2,'b',{get:()=>{throw 9;},enumerable:true})",
        )
        .unwrap();
    assert_eq!(
        realm.eval("Object.assign(o2,s2)"),
        Err(Error::Thrown(Value::Number(9.0)))
    );
    assert_eq!(realm.eval("o2.a"), Ok(Value::Number(1.0)));
    realm
        .eval("let o3={};Object.defineProperty(o3,'b',{set:()=>{throw 8;}})")
        .unwrap();
    assert_eq!(
        realm.eval("Object.assign(o3,{a:1,b:2,c:3})"),
        Err(Error::Thrown(Value::Number(8.0)))
    );
    assert_eq!(
        realm.eval("o3.a===1 && !Object.hasOwn(o3,'c')"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn descriptor_copy_preserves_accessors_and_hidden_properties_without_invoking_them() {
    check(
        "let o={},get=()=>{throw 9;};Object.defineProperties(o,{x:{get:get,enumerable:true},hidden:{value:3}});let ds=Object.getOwnPropertyDescriptors(o);ds.x.get===get && ds.x.set===undefined && ds.x.enumerable && !ds.x.configurable && ds.hidden.value===3 && !ds.hidden.enumerable && !Object.hasOwn(ds,'constructor')",
    );
    check(
        "let o={x:1},a=Object.getOwnPropertyDescriptors(o),b=Object.getOwnPropertyDescriptors(o);a!==b && a.x!==b.x && a.constructor===Object && a.x.constructor===Object && (a.x.value=2)===2 && b.x.value===1 && o.x===1",
    );
    check(
        "let o=Object.create(null,{x:{get:function(){return this;},enumerable:true,configurable:true}});let copy=Object.create(null,Object.getOwnPropertyDescriptors(o));copy!==o && copy.x===copy && Object.getOwnPropertyDescriptor(copy,'x').get===Object.getOwnPropertyDescriptor(o,'x').get",
    );
    check(
        "function f(a){a=7;return Object.getOwnPropertyDescriptors(arguments)['0'].value===7;}f(1)",
    );
}

#[test]
fn descriptor_results_define_data_properties_even_over_inherited_setters_and_proto_names() {
    check(
        "Object.defineProperty(Object.prototype,'x',{set:()=>{throw 9;}});let o=Object.create(null,{x:{value:7}});let ds=Object.getOwnPropertyDescriptors(o);ds.x.value===7 && Object.hasOwn(ds,'x') && Object.getOwnPropertyDescriptor(ds,'x').enumerable",
    );
    check(
        "let o={['__proto__']:7};let ds=Object.getOwnPropertyDescriptors(o);Object.getPrototypeOf(ds)===Object.prototype && Object.hasOwn(ds,'__proto__') && ds.__proto__.value===7 && Object.getOwnPropertyDescriptor(ds,'__proto__').writable && Object.getOwnPropertyDescriptor(ds,'__proto__').configurable",
    );
    check(
        "let ds=Object.getOwnPropertyDescriptors(true);ds.constructor===Object && !Object.hasOwn(ds,'valueOf')",
    );
    for source in [
        "Object.getOwnPropertyDescriptors()",
        "Object.getOwnPropertyDescriptors(null)",
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
    for source in [
        "Object.assign({},Object)",
        "Object.getOwnPropertyDescriptors(globalThis)",
        "Object.getOwnPropertyDescriptors(1n)",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}

#[test]
fn metadata_and_copied_edges_remain_valid_after_collection() {
    for (name, length) in [("assign", 2), ("getOwnPropertyDescriptors", 1)] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Object,'{name}');d.writable && !d.enumerable && d.configurable && d.value.name==='{name}' && d.value.length==={length}"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new Object.{name}()")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let C=Object;let o=C.assign({},{x:{value:7}});let ds=C.getOwnPropertyDescriptors(o);o=null;delete globalThis.Object").unwrap();
    realm.collect(100_000).unwrap();
    assert_eq!(
        realm.eval(
            "ds.x.value.value===7 && C.assign({},C.getOwnPropertyDescriptors({y:3})).y.value===3"
        ),
        Ok(Value::Boolean(true))
    );
}
