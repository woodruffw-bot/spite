use super::*;

#[test]
fn string_iterators_preserve_exact_code_point_boundaries() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor,key=S.iterator,i='A\\uD83D\\uDE00\\uD800x\\uDC00\\uD800\\uD800\\uDC00\\u0000e\\u0301'[key]();i.next().value==='A' && i.next().value==='\\uD83D\\uDE00' && i.next().value==='\\uD800' && i.next().value==='x' && i.next().value==='\\uDC00' && i.next().value==='\\uD800' && i.next().value==='\\uD800\\uDC00' && i.next().value==='\\u0000' && i.next().value==='e' && i.next().value==='\\u0301' && i.next().done",
    );
    check(
        &mut realm,
        "i='\\uDBFF\\uDFFF'[key]();i.next().value==='\\uDBFF\\uDFFF' && i.next().done && ''[key]().next().done",
    );
    check(
        &mut realm,
        "i='a'[key]();let a=i.next(),b=i.next(),c=i.next();a.value==='a' && !a.done && b.done && b.value===undefined && c.done && c.value===undefined && a!==b && b!==c && i!==('a'[key]())",
    );
}

#[test]
fn string_iteration_converts_once_at_creation_and_ignores_arguments() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let key=s.constructor.iterator,make=''[key],calls=0,log='',o={[convert]:(hint)=>{calls++;log+=hint;return 'xy';}},i=make.call(o,{toString:()=>{throw 7;}});o[convert]=()=>{throw 8;};calls===1 && log==='string' && i.next({valueOf:()=>{throw 9;}}).value==='x' && i.next().value==='y' && i.next().done && calls===1",
    );
    check(
        &mut realm,
        "let a=make.call(12),b=make.call(true),c=make.call(34n);a.next().value==='1' && a.next().value==='2' && b.next().value==='t' && c.next().value==='3' && c.next().value==='4' && make.call(new String('z')).next().value==='z'",
    );
    check(
        &mut realm,
        "let order='',j=make.call({toString:()=>{order+='s';return {};},valueOf:()=>{order+='v';return 'q';}});order==='sv' && j.next().value==='q'",
    );
}

#[test]
fn string_iterator_coercion_failures_are_immediate() {
    let mut realm = realm_with_symbols();
    realm.eval("let make=''[s.constructor.iterator]").unwrap();
    for receiver in [
        "null",
        "undefined",
        "s",
        "Object(s)",
        "{[convert]:()=>({})}",
    ] {
        assert!(
            matches!(
                realm.eval(&format!("make.call({receiver})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{receiver}"
        );
    }
    assert_eq!(
        realm.eval("make.call({toString:()=>{throw 7;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
}

#[test]
fn string_iterator_prototypes_and_descriptors_are_standard() {
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let S=s.constructor,key=S.iterator,make=String.prototype[key],i='x'[key](),p=Object.getPrototypeOf(i),base=Object.getPrototypeOf(p),d=Object.getOwnPropertyDescriptor(String.prototype,key);make.name==='[Symbol.iterator]' && make.length===0 && d.value===make && d.writable && !d.enumerable && d.configurable && base===Object.getPrototypeOf(Object.getPrototypeOf([].values())) && i[key]()===i",
    );
    check(
        &mut realm,
        "d=Object.getOwnPropertyDescriptor(p,'next');d.value===i.next && d.value.name==='next' && d.value.length===0 && d.writable && !d.enumerable && d.configurable",
    );
    check(
        &mut realm,
        "d=Object.getOwnPropertyDescriptor(p,S.toStringTag);d.value==='String Iterator' && !d.writable && !d.enumerable && d.configurable && Object.prototype.toString.call(i)==='[object String Iterator]'",
    );
    check(
        &mut realm,
        "let r=i.next();d=Object.getOwnPropertyDescriptor(r,'value');d.value==='x' && d.writable && d.enumerable && d.configurable && Object.getPrototypeOf(r)===Object.prototype",
    );
    check(
        &mut realm,
        "delete String.prototype[key];String.prototype[key]===undefined && make.call('a').next().value==='a'",
    );
    for source in ["new make", "new i.next"] {
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
fn string_next_requires_its_own_brand_independent_of_prototypes() {
    for receiver in [
        "null",
        "undefined",
        "''",
        "1",
        "s",
        "{}",
        "[]",
        "[].values()",
        "p",
        "Object.create(i)",
    ] {
        let mut realm = realm_with_symbols();
        realm
            .eval("let i='x'[s.constructor.iterator](),p=Object.getPrototypeOf(i),next=i.next")
            .unwrap();
        assert!(
            matches!(
                realm.eval(&format!("next.call({receiver})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{receiver}"
        );
    }
    let mut realm = realm_with_symbols();
    check(
        &mut realm,
        "let i='x'[s.constructor.iterator](),next=i.next;Object.setPrototypeOf(i,null);next.call(i).value==='x' && next.call(i).done",
    );
    assert!(matches!(
        realm.eval("[].values().next.call(i)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn string_iterators_retain_captured_text_without_retaining_the_original_receiver() {
    let mut realm = realm_with_symbols();
    let Value::Object(source) = realm
        .eval("let o={toString:()=> 'xy'},i=String.prototype[s.constructor.iterator].call(o);o")
        .unwrap()
    else {
        panic!("source");
    };
    realm.eval("o=null").unwrap();
    realm.collect(30_000).unwrap();
    assert!(matches!(
        realm.objects.inspect(&source),
        Err(crate::object::Error::Heap(spite_heap::Error::StaleHandle))
    ));
    check(
        &mut realm,
        "i.next().value==='x' && i.next().value==='y' && i.next().done",
    );
    realm.collect(30_000).unwrap();
    check(&mut realm, "i.next().done");
}

#[test]
fn recursive_string_conversion_obeys_host_limits_and_restores_call_state() {
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let mut realm = realm_with_symbols();
            realm
                .eval("let make=''[s.constructor.iterator],flag=0,o={toString:()=>make.call(o)}")
                .unwrap();
            assert!(matches!(
                realm.eval("try{make.call(o);}catch{flag=1;}finally{flag=2;}"),
                Err(Error::Limit { .. })
            ));
            check(&mut realm, "flag===0 && make.call('x').next().value==='x'");
        })
        .unwrap()
        .join()
        .unwrap();
}
