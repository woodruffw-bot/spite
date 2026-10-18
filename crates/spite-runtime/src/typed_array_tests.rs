//! TypedArray essential internal methods before public constructors (10.4.5).

use crate::{Error, ObjectHandle, Realm, Value, binary_data::ElementType, object::Budget};
use spite_core::{JsString, Span};

// Construct only test fixtures through the internal slot API. These globals are
// ordinary test bindings and do not expose a TypedArray constructor to programs.
fn fixture(
    element: ElementType,
    byte_length: u64,
    maximum: Option<u64>,
    byte_offset: u64,
    array_length: Option<u64>,
) -> (Realm, ObjectHandle) {
    let mut realm = Realm::default();
    realm.eval("").unwrap();
    let object_prototype = realm.intrinsics.as_ref().unwrap().object_prototype.clone();
    let buffer_prototype = realm
        .intrinsics
        .as_ref()
        .unwrap()
        .array_buffer
        .prototype
        .clone();
    let prototype = realm.objects.create(Some(&object_prototype)).unwrap();
    let buffer = realm
        .objects
        .create_array_buffer(
            &buffer_prototype,
            byte_length,
            maximum,
            &mut Budget::with_work_limit(None),
        )
        .unwrap();
    let view = realm
        .objects
        .create_typed_array(&prototype, &buffer, element, byte_offset, array_length)
        .unwrap();
    let global = realm.global_object.clone().unwrap();
    for (name, object) in [
        ("buffer", buffer),
        ("prototype", prototype),
        ("view", view.clone()),
    ] {
        realm
            .define_builtin_property(&global, name, Value::Object(object), true, Span::new(0, 0))
            .unwrap();
    }
    (realm, view)
}

fn check(realm: &mut Realm, body: &str) {
    let source = format!("(()=>{{{body}}})()");
    assert_eq!(realm.eval(&source), Ok(Value::Boolean(true)), "{source}");
}

#[test]
fn canonical_numeric_keys_block_prototypes_while_other_spellings_remain_ordinary() {
    let (mut realm, _) = fixture(ElementType::Uint8, 2, None, 0, Some(2));
    check(
        &mut realm,
        "let invalid=['-0','NaN','Infinity','-Infinity','-1','0.5','2','9007199254740992','1e+21'];for(let key of invalid){Object.defineProperty(prototype,key,{get(){throw 7;},set(){throw 8;},configurable:true});if(view[key]!==undefined || key in view || Object.getOwnPropertyDescriptor(view,key)!==undefined || !Reflect.deleteProperty(view,key))return false;}let receiver;Object.defineProperty(prototype,'01',{get(){receiver=this;return 7;},configurable:true});prototype['1.0']=8;prototype['+0']=9;prototype['1e21']=10;if(view['01']!==7 || receiver!==view || view['1.0']!==8 || view['+0']!==9 || view['1e21']!==10 || !('01' in view))return false;view['1.0']=11;return Object.hasOwn(view,'1.0') && view['1.0']===11 && prototype['1.0']===8 && view[0]===0 && view[1]===0;",
    );
}

#[test]
fn same_receiver_set_converts_even_invalid_indices_and_uses_post_conversion_bounds() {
    let (mut realm, _) = fixture(ElementType::Uint8, 4, Some(8), 0, Some(4));
    check(
        &mut realm,
        "let hits=0,value={valueOf(){hits++;return 9;}};for(let key of ['-0','NaN','Infinity','-1','0.5','4']){if(!Reflect.set(view,key,value))return false;}if(hits!==6 || view[0]!==0 || Object.hasOwn(view,'-0'))return false;buffer.resize(1);if(!Reflect.set(view,'0',{valueOf(){buffer.resize(4);return 7;}}) || view[0]!==7)return false;if(!Reflect.set(view,'0',{valueOf(){buffer.resize(0);return 9;}}))return false;buffer.resize(4);if(view[0]!==0)return false;view[0]=3;let moved;if(!Reflect.set(view,'0',{valueOf(){moved=buffer.transfer();return 8;}}))return false;let thrown=false;try{Reflect.set(view,'0',{valueOf(){throw 11;}});}catch(e){thrown=e===11;}return thrown && view[0]===undefined && new DataView(moved).getUint8(0)===3;",
    );
}

#[test]
fn distinct_receiver_set_skips_invalid_indices_and_preserves_ordinary_receiver_rules() {
    let (mut realm, _) = fixture(ElementType::Uint8, 2, None, 0, Some(2));
    check(
        &mut realm,
        // Edition 17 10.4.5.6 first checks the target's numeric index. An invalid
        // index reports success before inspecting a distinct receiver.
        "view[0]=3;let hits=0,value={valueOf(){hits++;throw 7;}},receiver={};Object.defineProperty(receiver,'-0',{value:1,writable:false});if(!Reflect.set(view,'-0',value,receiver) || !Reflect.set(view,'9',value,null) || hits!==0 || receiver['-0']!==1)return false;if(!Reflect.set(view,'0',value,receiver))return false;let d=Object.getOwnPropertyDescriptor(receiver,'0');if(d.value!==value || !d.writable || !d.enumerable || !d.configurable || view[0]!==3 || hits!==0)return false;let setterHits=0,accessor={};Object.defineProperty(accessor,'0',{set(){setterHits++;},configurable:true});let readonly={};Object.defineProperty(readonly,'0',{value:1,writable:false});if(Reflect.set(view,'0',value,accessor) || Reflect.set(view,'0',value,readonly) || Reflect.set(view,'0',value,null))return false;let child=Object.create(view);child[0]=value;child[9]=value;return Object.hasOwn(child,'0') && child[0]===value && !Object.hasOwn(child,'9') && setterHits===0 && hits===0 && view[0]===3;",
    );
}

#[test]
fn define_rejects_invalid_indices_and_incompatible_descriptors_without_numeric_coercion() {
    let (mut realm, _) = fixture(ElementType::Uint8, 2, None, 0, Some(2));
    check(
        &mut realm,
        "view[0]=3;let hits=0,value={valueOf(){hits++;return 7;}};for(let descriptor of [{value,configurable:false},{value,enumerable:false},{value,writable:false},{get(){hits++;return 7;}},{set(v){hits++;}}]){if(Reflect.defineProperty(view,'0',descriptor))return false;}for(let key of ['-0','NaN','Infinity','-1','0.5','2']){if(Reflect.defineProperty(view,key,{value}))return false;}if(hits!==0 || view[0]!==3 || !Reflect.defineProperty(view,'0',{}))return false;let ok=Reflect.defineProperty(view,'0',{value,writable:true,enumerable:true,configurable:true});let d=Object.getOwnPropertyDescriptor(view,'0');return ok && hits===1 && d.value===7 && d.writable && d.enumerable && d.configurable;",
    );
}

#[test]
fn define_keeps_its_success_result_when_value_conversion_invalidates_the_element() {
    let (mut realm, _) = fixture(ElementType::Uint8, 4, Some(8), 0, Some(4));
    check(
        &mut realm,
        "view[0]=3;let ok=Reflect.defineProperty(view,'0',{value:{valueOf(){buffer.resize(0);return 7;}}});if(!ok || Object.hasOwn(view,'0'))return false;let hits=0;if(Reflect.defineProperty(view,'0',{value:{valueOf(){hits++;buffer.resize(4);return 8;}}}) || hits!==0 || buffer.byteLength!==0)return false;buffer.resize(4);if(view[0]!==0)return false;view[0]=9;let moved;ok=Reflect.defineProperty(view,'0',{value:{valueOf(){moved=buffer.transfer();return 1;}}});return ok && view[0]===undefined && new DataView(moved).getUint8(0)===9;",
    );
}

#[test]
fn fixed_and_auto_witnesses_drop_partial_elements_and_recover_after_regrowth() {
    let (mut realm, _) = fixture(ElementType::Uint16, 6, Some(10), 2, Some(2));
    let buffer = match realm.eval("buffer").unwrap() {
        Value::Object(buffer) => buffer,
        _ => unreachable!(),
    };
    let prototype = realm.intrinsics.as_ref().unwrap().object_prototype.clone();
    let auto = realm
        .objects
        .create_typed_array(&prototype, &buffer, ElementType::Uint16, 2, None)
        .unwrap();
    let global = realm.global_object.clone().unwrap();
    realm
        .define_builtin_property(&global, "auto", Value::Object(auto), true, Span::new(0, 0))
        .unwrap();
    check(
        &mut realm,
        "view[0]=7;view[1]=9;if(auto[0]!==7 || auto[1]!==9)return false;buffer.resize(5);if(view[0]!==undefined || '0' in view || Reflect.ownKeys(view).length!==0 || auto[0]!==7 || auto[1]!==undefined || Reflect.ownKeys(auto).join()!=='0')return false;buffer.resize(2);if(Reflect.ownKeys(auto).length!==0 || auto[0]!==undefined)return false;buffer.resize(1);if(auto[0]!==undefined || '0' in auto)return false;buffer.resize(8);return Reflect.ownKeys(view).join()==='0,1' && Reflect.ownKeys(auto).join()==='0,1,2' && view[0]===0 && auto[2]===0;",
    );
}

#[test]
fn reflection_reports_element_descriptors_but_deletion_rejects_live_indices() {
    let (mut realm, _) = fixture(ElementType::Uint8, 3, None, 0, Some(3));
    check(
        &mut realm,
        "view[1]=7;let d=Object.getOwnPropertyDescriptor(view,'1');if(d.value!==7 || !d.writable || !d.enumerable || !d.configurable || !Object.hasOwn(view,'1') || !('1' in view) || Reflect.deleteProperty(view,'1'))return false;let caught=false;try{(()=>{'use strict';delete view[1];})();}catch(e){caught=e instanceof TypeError;}if(!caught)return false;let first=Symbol('first'),last=Symbol('last');view.note=1;view['01']=2;view[first]=3;view.later=4;view[last]=5;delete view.note;view.note=6;let keys=Reflect.ownKeys(view);if(keys.length!==8 || keys.slice(0,6).join()!=='0,1,2,01,later,note' || keys[6]!==first || keys[7]!==last)return false;buffer.transfer();keys=Reflect.ownKeys(view);return keys.length===5 && keys.slice(0,3).join()==='01,later,note' && keys[3]===first && keys[4]===last && view[1]===undefined && !('1' in view) && Reflect.deleteProperty(view,'1');",
    );
}

#[test]
fn prevent_extensions_allows_fixed_buffers_and_rejects_every_resizable_view() {
    let (mut fixed, _) = fixture(ElementType::Uint8, 2, None, 0, Some(2));
    check(
        &mut fixed,
        "if(!Reflect.preventExtensions(view) || Object.isExtensible(view))return false;view[0]=7;if(view[0]!==7 || Reflect.defineProperty(view,'note',{value:1}))return false;buffer.transfer();return Reflect.preventExtensions(view) && !Object.isExtensible(view) && Reflect.ownKeys(view).length===0;",
    );
    for length in [Some(2), None, Some(0)] {
        let (mut resizable, _) = fixture(ElementType::Uint8, 2, Some(4), 0, length);
        check(
            &mut resizable,
            "if(Reflect.preventExtensions(view) || !Object.isExtensible(view))return false;let caught=false;try{Object.preventExtensions(view);}catch(e){caught=e instanceof TypeError;}if(!caught || !Object.isExtensible(view))return false;buffer.transfer();return !Reflect.preventExtensions(view) && Object.isExtensible(view);",
        );
    }
}

#[test]
fn uint8_clamped_elements_use_even_halfway_rounding_and_share_buffer_bytes() {
    let (mut realm, _) = fixture(ElementType::Uint8Clamped, 8, None, 0, Some(8));
    check(
        &mut realm,
        "let input=[-1,0.5,1.5,2.5,3.5,254.5,255.5,Infinity],expected=[0,0,2,2,4,254,255,255],bytes=new DataView(buffer);for(let i=0;i<8;i++){view[i]=input[i];if(view[i]!==expected[i] || bytes.getUint8(i)!==expected[i])return false;}view[0]=NaN;bytes.setUint8(1,9);return view[0]===0 && view[1]===9;",
    );
}

#[test]
fn bigint_elements_convert_even_invalid_sets_but_not_rejected_definitions() {
    let (mut realm, _) = fixture(ElementType::BigUint64, 8, None, 0, Some(1));
    check(
        &mut realm,
        "let caught=false;try{Reflect.set(view,'9',1);}catch(e){caught=e instanceof TypeError;}if(!caught || Reflect.defineProperty(view,'9',{value:1}))return false;view[0]=-1n;if(view[0]!==18446744073709551615n)return false;view[0]='1';if(view[0]!==1n)return false;let hits=0;if(!Reflect.set(view,'NaN',{valueOf(){hits++;return 9n;}}) || hits!==1)return false;let receiver={};if(!Reflect.set(view,'0',1,receiver) || receiver[0]!==1)return false;buffer.transfer();caught=false;try{view[0]=Symbol();}catch(e){caught=e instanceof TypeError;}return caught && view[0]===undefined;",
    );
}

#[test]
fn opted_in_bit_and_work_limits_stay_host_failures_without_disabling_index_probes() {
    let (mut realm, _) = fixture(ElementType::BigUint64, 8, None, 0, Some(1));
    realm
        .eval(&format!(
            "new DataView(buffer).setBigUint64(0,256n,{});let flag=0",
            cfg!(target_endian = "little")
        ))
        .unwrap();
    realm.limits.max_bigint_bits = Some(8);
    check(
        &mut realm,
        "return '0' in view && Reflect.has(view,'0') && !Reflect.deleteProperty(view,'0') && Reflect.ownKeys(view).join()==='0';",
    );
    assert!(matches!(
        realm.eval("try{view[0];}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert!(matches!(
        realm.eval("Object.getOwnPropertyDescriptor(view,'0')"),
        Err(Error::Limit { .. })
    ));
    realm.limits.max_bigint_bits = None;
    check(&mut realm, "return view[0]===256n;");

    let (mut realm, view) = fixture(ElementType::Uint8, 10_000, None, 0, Some(10_000));
    realm.remaining_steps = Some(100);
    assert!(matches!(
        realm.own_property_keys(&view, Span::new(0, 0)),
        Err(Error::Limit { .. })
    ));
    realm.remaining_steps = None;
    assert_eq!(
        realm.get_property(&view, &JsString::from("0"), Span::new(0, 0)),
        Ok(Value::Number(0.0))
    );
}

#[test]
fn an_escaped_typed_array_keeps_its_buffer_alive_through_collection() {
    let (mut realm, view) = fixture(ElementType::Uint8, 4, Some(8), 0, Some(4));
    realm
        .eval("view[0]=7;view[3]=9;buffer=undefined;prototype=undefined")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "return view[0]===7 && view[3]===9 && ArrayBuffer.isView(view);",
    );
    realm.eval("view=undefined").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert!(matches!(
        realm.objects.inspect(&view),
        Err(crate::object::Error::Heap(spite_heap::Error::StaleHandle))
    ));
}

#[test]
fn borrowed_array_iterators_ignore_public_length_and_complete_permanently_after_shrink() {
    for method in ["values", "keys", "entries"] {
        let (mut realm, _) = fixture(ElementType::Uint8, 2, Some(8), 0, None);
        let source = r"
            let method='@METHOD@';
            Object.defineProperty(view,'length',{get(){throw 7;}});
            function matches(result,index,value){
                if(result.done)return false;
                if(method==='keys')return result.value===index;
                if(method==='values')return result.value===value;
                return result.value[0]===index && result.value[1]===value;
            }
            view[0]=10;view[1]=11;
            let iterator=Array.prototype[method].call(view);
            if(!matches(iterator.next(),0,10))return false;
            buffer.resize(4);view[2]=12;view[3]=13;
            if(!matches(iterator.next(),1,11) || !matches(iterator.next(),2,12))return false;
            buffer.resize(3);
            let end=iterator.next();
            if(!end.done || end.value!==undefined)return false;
            buffer.resize(8);view[3]=23;
            let afterGrowth=iterator.next();
            buffer.transfer();
            let afterDetach=iterator.next();
            return afterGrowth.done && afterGrowth.value===undefined &&
                   afterDetach.done && afterDetach.value===undefined;
        "
        .replace("@METHOD@", method);
        check(&mut realm, &source);
    }
}

#[test]
fn borrowed_array_iterator_bounds_failures_preserve_position_and_is_view_identity() {
    for method in ["values", "keys", "entries"] {
        for length in [Some(2), None] {
            let (mut realm, _) = fixture(ElementType::Uint8, 4, Some(8), 2, length);
            let source = r"
                let method='@METHOD@';
                Object.defineProperty(view,'length',{get(){throw 7;}});
                function matches(result,index,value){
                    if(result.done)return false;
                    if(method==='keys')return result.value===index;
                    if(method==='values')return result.value===value;
                    return result.value[0]===index && result.value[1]===value;
                }
                if(!ArrayBuffer.isView(view) || ArrayBuffer.isView(Object.create(view)) ||
                   ArrayBuffer.isView(prototype) || ArrayBuffer.isView(buffer))return false;
                view[0]=10;view[1]=11;
                let iterator=Array.prototype[method].call(view);
                if(!matches(iterator.next(),0,10))return false;
                buffer.resize(1);
                let failed=false;
                try{iterator.next();}catch(e){failed=e instanceof TypeError;}
                if(!failed || !ArrayBuffer.isView(view))return false;
                buffer.resize(6);view[1]=21;
                if(!matches(iterator.next(),1,21))return false;
                buffer.transfer();
                failed=false;
                try{iterator.next();}catch(e){failed=e instanceof TypeError;}
                return failed && ArrayBuffer.isView(view);
            "
            .replace("@METHOD@", method);
            check(&mut realm, &source);
        }
    }
}
