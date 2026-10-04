use super::*;

fn realm_with_tag_accessors() -> Realm {
    let mut realm = realm_with_symbols();
    realm.eval("let S=s.constructor,key=S.toStringTag,base=Object.getPrototypeOf(Object.getPrototypeOf([].values())),d=Object.getOwnPropertyDescriptor(base,key),get=d.get,set=d.set").unwrap();
    realm
}

#[test]
fn iterator_tag_has_generic_getter_and_standard_accessor_attributes() {
    let mut realm = realm_with_tag_accessors();
    check(
        &mut realm,
        "!d.enumerable && d.configurable && get.length===0 && get.name==='get [Symbol.toStringTag]' && set.length===1 && set.name==='set [Symbol.toStringTag]' && !Object.hasOwn(get,'prototype') && !Object.hasOwn(set,'prototype')",
    );
    check(
        &mut realm,
        "base[key]==='Iterator' && get.call(undefined)==='Iterator' && get.call(null)==='Iterator' && get.call(s)==='Iterator' && get.call({toString:()=>{throw 7;}})==='Iterator' && Object.prototype.toString.call(base)==='[object Iterator]'",
    );
    for source in ["new get", "new set"] {
        assert!(matches!(
            realm.eval(source),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn tag_setter_rejects_primitives_and_home_even_after_deletion() {
    let mut realm = realm_with_tag_accessors();
    for receiver in ["undefined", "null", "true", "1", "1n", "'x'", "s", "base"] {
        assert!(
            matches!(
                realm.eval(&format!("set.call({receiver},'x')")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{receiver}"
        );
    }
    check(
        &mut realm,
        "let caught=false;try{base[key]='x';}catch(e){caught=e instanceof TypeError;}caught && base[key]==='Iterator'",
    );
    check(
        &mut realm,
        "delete base[key];base[key]===undefined && get.call(base)==='Iterator'",
    );
    assert!(matches!(
        realm.eval("set.call(base,'x')"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn tag_setter_creates_own_data_properties_without_inherited_lookups() {
    let mut realm = realm_with_tag_accessors();
    check(
        &mut realm,
        "let child=Object.create(base),v={toString:()=>{throw 7;}};child[key]=v;d=Object.getOwnPropertyDescriptor(child,key);d.value===v && d.writable && d.enumerable && d.configurable && base[key]==='Iterator'",
    );
    check(
        &mut realm,
        "let parent={};Object.defineProperty(parent,key,{get:()=>{throw 7;},set:()=>{throw 8;}});let o=Object.create(parent);set.call(o,'x')===undefined && Object.hasOwn(o,key) && o[key]==='x'",
    );
    check(
        &mut realm,
        "let frozen={};Object.defineProperty(frozen,key,{value:7});o=Object.create(frozen);set.call(o,'y')===undefined && o[key]==='y' && frozen[key]===7",
    );
    check(
        &mut realm,
        "o=Object.create(null);set.call(o)===undefined && Object.hasOwn(o,key) && o[key]===undefined",
    );
    check(
        &mut realm,
        "Object.defineProperty(base,key,{configurable:false});let inheritor=Object.create(base);inheritor[key]='Child';inheritor[key]==='Child' && base[key]==='Iterator'",
    );
    assert!(matches!(
        realm.eval("set.call(Object.preventExtensions({}),'x')"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn tag_setter_uses_strict_set_for_existing_own_properties() {
    let mut realm = realm_with_tag_accessors();
    check(
        &mut realm,
        "let o={};Object.defineProperty(o,key,{value:'a',writable:true});Object.preventExtensions(o);set.call(o,'b');d=Object.getOwnPropertyDescriptor(o,key);d.value==='b' && d.writable && !d.enumerable && !d.configurable",
    );
    check(
        &mut realm,
        "let receiver,value,count=0,a={};Object.defineProperty(a,key,{get:()=>{throw 7;},set:function(v){'use strict';receiver=this;value=v;count++;}});let v={};set.call(a,v)===undefined && receiver===a && value===v && count===1",
    );
    for descriptor in ["{value:7}", "{get:()=>7}"] {
        assert!(matches!(
            realm.eval(&format!(
                "let b={{}};Object.defineProperty(b,key,{descriptor});set.call(b,'x')"
            )),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        // The failing Script has already installed its lexical binding.
        realm = realm_with_tag_accessors();
    }
    assert_eq!(
        realm.eval("let o={};Object.defineProperty(o,key,{set:()=>{throw 7;}});set.call(o,'x')"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
}

#[test]
fn inherited_iterator_tags_remain_live_after_overrides_and_collection() {
    let mut realm = realm_with_tag_accessors();
    check(
        &mut realm,
        "let i=[].values(),p=Object.getPrototypeOf(i);delete p[key];i[key]==='Iterator' && Object.prototype.toString.call(i)==='[object Iterator]' && (i[key]='Custom',Object.prototype.toString.call(i)==='[object Custom]')",
    );
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "get.call(i)==='Iterator' && set.call(i,'Other')===undefined && i[key]==='Other' && base[key]==='Iterator'",
    );
}

#[test]
fn recursive_tag_setters_obey_host_limits_and_restore_call_state() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut realm = realm_with_tag_accessors();
            realm
                .eval("let o={},flag=0;Object.defineProperty(o,key,{set:()=>set.call(o,'x')})")
                .unwrap();
            assert!(matches!(
                realm.eval("try{set.call(o,'x');}catch{flag=1;}finally{flag=2;}"),
                Err(Error::Limit { .. })
            ));
            check(&mut realm, "flag===0 && set.call({},'x')===undefined");
        })
        .unwrap()
        .join()
        .unwrap();
}
