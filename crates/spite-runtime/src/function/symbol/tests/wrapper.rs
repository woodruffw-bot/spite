use super::*;
use spite_core::WellKnownSymbol;

#[test]
fn fresh_symbols_preserve_description_conversion_and_undefined_distinction() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor;typeof S==='function' && S.name==='Symbol' && S.length===0",
    );
    check(
        &mut realm,
        "let a=S(),b=S(undefined),c=S(''),d=S('name');a!==b && a.description===undefined && b.description===undefined && c.description==='' && d.description==='name' && d!==S('name')",
    );
    check(
        &mut realm,
        "S(null).description==='null' && S(3).description==='3' && S(4n).description==='4' && S(true).description==='true'",
    );
    check(
        &mut realm,
        "let log='',k={[convert]:(hint)=>{log+=hint;return 'text';}};S(k).description==='text' && log==='string'",
    );
    check(
        &mut realm,
        "S('\\uD800\\u0000\\uDC00').description==='\\uD800\\u0000\\uDC00' && S.call({},'x').description==='x'",
    );
    assert!(matches!(
        realm.eval("S(s)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(
        realm.eval("S({toString:()=>{throw 7;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    check(
        &mut realm,
        "let converted=false,caught=false;try{new S({toString:()=>{converted=true;return 'x';}});}catch(e){caught=e instanceof TypeError;}caught && !converted",
    );
    assert!(matches!(
        realm.eval("new (S.bind(null))()"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn wrappers_keep_identity_and_brand_independently_of_their_prototype() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor,box=Object(s);typeof box==='object' && box!==Object(s) && Object(box)===box && Object.getPrototypeOf(box)===S.prototype && box.valueOf()===s && box==s && s==box && box!==s && box instanceof S && !(s instanceof S)",
    );
    check(
        &mut realm,
        "box.toString()==='Symbol(name)' && s.toString()==='Symbol(name)' && s.valueOf()===s && s.description==='name' && box.description==='name'",
    );
    check(
        &mut realm,
        "box[convert]({toString:()=>{throw 7;}})===s && s[convert]()===s && Object.prototype.toString.call(box)==='[object Symbol]' && Object.prototype.toString.call(s)==='[object Symbol]'",
    );
    check(
        &mut realm,
        "[s].toLocaleString()==='Symbol(name)' && Object.getOwnPropertyDescriptors(Object(s)).x===undefined",
    );
    check(
        &mut realm,
        "let valueOf=S.prototype.valueOf,toString=S.prototype.toString;Object.setPrototypeOf(box,null);valueOf.call(box)===s && toString.call(box)==='Symbol(name)'",
    );
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "valueOf.call(box)===s && s.constructor===S && S.iterator===S.iterator",
    );
}

#[test]
fn symbol_prototype_is_ordinary_and_brand_checks_never_coerce() {
    for value in [
        "undefined",
        "null",
        "false",
        "0",
        "1n",
        "'x'",
        "{}",
        "S.prototype",
        "Object.create(S.prototype)",
        "{[convert]:()=>{throw 9;},valueOf:()=>{throw 8;}}",
    ] {
        let mut realm = realm_with_symbols();
        realm.eval("let S=s.constructor,proto=S.prototype,description=Object.getOwnPropertyDescriptor(proto,'description').get").unwrap();
        for method in [
            "proto.toString",
            "proto.valueOf",
            "proto[convert]",
            "description",
        ] {
            assert!(
                matches!(
                    realm.eval(&format!("{method}.call({value})")),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{method}, {value}"
            );
        }
    }
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let proto=s.constructor.prototype;Object.getPrototypeOf(proto)===Object.prototype && Object.prototype.toString.call(proto)==='[object Symbol]'",
    );
    check(
        &mut realm,
        "delete proto[tag];Object.prototype.toString.call(Object(s))==='[object Object]' && Object.prototype.toString.call(s)==='[object Object]'",
    );
}

#[test]
fn wrapper_coercion_uses_to_primitive_and_rejects_implicit_strings_and_numbers() {
    for expression in [
        "String(Object(s))",
        "Number(Object(s))",
        "''+Object(s)",
        "+Object(s)",
        "new String(s)",
    ] {
        assert!(
            matches!(
                realm_with_symbols().eval(expression),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{expression}"
        );
    }
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let box=Object(s),o={[s]:7};o[box]===7 && String(s)==='Symbol(name)'",
    );
    check(
        &mut realm,
        "Object.defineProperty(box,convert,{value:()=> 'custom'});String(box)==='custom' && s.toString()==='Symbol(name)'",
    );
    check(
        &mut realm,
        "let proto=s.constructor.prototype;delete proto[convert];String(Object(s))==='Symbol(name)'",
    );
}

#[test]
fn symbol_intrinsic_descriptors_and_methods_have_standard_attributes() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor,p=S.prototype,d=Object.getOwnPropertyDescriptor(S,'prototype');d.value===p && !d.writable && !d.enumerable && !d.configurable",
    );
    check(
        &mut realm,
        "d=Object.getOwnPropertyDescriptor(p,'description');d.set===undefined && d.get.length===0 && d.get.name==='get description' && !d.enumerable && d.configurable && d.get.call(unnamed)===undefined && d.get.call(empty)==='' && d.get.call(raw)==='\\uD800\\u0000\\uDC00'",
    );
    check(
        &mut realm,
        "d=Object.getOwnPropertyDescriptor(p,convert);!d.writable && !d.enumerable && d.configurable && d.value.length===1 && d.value.name==='[Symbol.toPrimitive]' && !Object.hasOwn(d.value,'prototype')",
    );
    check(
        &mut realm,
        "d=Object.getOwnPropertyDescriptor(p,tag);d.value==='Symbol' && !d.writable && !d.enumerable && d.configurable",
    );
    for name in ["constructor", "toString", "valueOf"] {
        check(
            &mut realm,
            &format!(
                "d=Object.getOwnPropertyDescriptor(p,'{name}');d.writable && !d.enumerable && d.configurable"
            ),
        );
    }
    for key in WellKnownSymbol::ALL {
        let name = key.name();
        check(
            &mut realm,
            &format!(
                "d=Object.getOwnPropertyDescriptor(S,'{name}');typeof d.value==='symbol' && d.value.description==='Symbol.{name}' && !d.writable && !d.enumerable && !d.configurable"
            ),
        );
        assert_eq!(
            realm.eval(&format!("S.{name}")),
            Ok(Value::Symbol(key.symbol()))
        );
    }
}

#[test]
fn symbol_primitives_keep_strict_getter_receivers_and_sloppy_methods_box() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let p=s.constructor.prototype,seen,assigned;Object.defineProperty(p,'strictGet',{get:function(){'use strict';seen=this;return this;},set:function(v){'use strict';seen=this;assigned=v;}});s.strictGet===s && seen===s",
    );
    check(&mut realm, "s.strictGet=7;seen===s && assigned===7");
    check(
        &mut realm,
        "Object.defineProperty(p,'sloppyGet',{get:function(){return this;}});s.sloppyGet!==s.sloppyGet && s.sloppyGet.valueOf()===s",
    );
    check(&mut realm, "s.description='other';s.description==='name'");
    assert!(matches!(
        realm.eval("'use strict';s.description='other'"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn description_output_and_wrapper_allocation_obey_host_limits() {
    let mut realm = realm_with_symbols();
    realm
        .eval("let S=s.constructor,description='long text',flag=0,g=Object.getOwnPropertyDescriptor(S.prototype,'description').get")
        .unwrap();
    realm.limits.max_string_units = Some(8);
    assert!(matches!(
        realm.eval("try{S(description);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    check(&mut realm, "flag===0");
    realm.limits.max_string_units = Some(1_048_576);
    realm.eval("let large=S(description)").unwrap();
    realm.limits.max_string_units = Some(8);
    assert!(matches!(
        realm.eval("g.call(large)"),
        Err(Error::Limit { .. })
    ));
    // Identity extraction and boxing do not copy the description text.
    check(
        &mut realm,
        "large.valueOf()===large && Object(large).valueOf()===large",
    );
}
