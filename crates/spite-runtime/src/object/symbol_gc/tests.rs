//! Joint object/Symbol ephemeron, ownership and failure regressions.

use crate::object::{Budget, DataDescriptor, Error as ObjectError, Objects};
use crate::{Realm, Value};
use spite_core::{JsString, JsSymbol, WeakJsSymbol};
use spite_heap::Handle;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::mpsc,
    thread,
};

fn unlimited() -> Budget {
    Budget::with_work_limit(None)
}

fn data(value: Value) -> DataDescriptor {
    DataDescriptor {
        value: Some(value),
        writable: Some(true),
        enumerable: Some(true),
        configurable: Some(true),
    }
}

fn object(value: Value) -> Handle {
    let Value::Object(value) = value else {
        panic!("expected object")
    };
    value
}

fn symbol(value: Value) -> JsSymbol {
    let Value::Symbol(value) = value else {
        panic!("expected Symbol")
    };
    value
}

fn empty_map(objects: &mut Objects) -> Handle {
    let prototype = objects.create(None).unwrap();
    objects.create_weak_map(&prototype).unwrap()
}

fn symbol_entries(objects: &Objects, map: &Handle) -> usize {
    objects
        .inspect(map)
        .unwrap()
        .weak_map
        .as_ref()
        .unwrap()
        .symbol_entries
        .len()
}

fn object_entries(objects: &Objects, map: &Handle) -> usize {
    objects
        .inspect(map)
        .unwrap()
        .weak_map
        .as_ref()
        .unwrap()
        .entries
        .len()
}

fn stale(objects: &Objects, value: &Handle) {
    assert!(matches!(
        objects.inspect(value),
        Err(ObjectError::Heap(spite_heap::Error::StaleHandle))
    ));
}

// Rooted map --weak Symbol key--> value --ordinary property--> the same Symbol.
// Returning only unrooted object Handles and a Symbol Weak adds no graph root.
fn install_closed_cycle(objects: &mut Objects) -> (Handle, Handle, WeakJsSymbol) {
    let map = empty_map(objects);
    let value = objects.create(None).unwrap();
    let key = JsSymbol::new(Some(JsString::from("cycle")));
    let weak = key.downgrade();
    assert!(
        objects
            .define(
                &value,
                "back",
                data(Value::Symbol(key.clone())),
                &mut unlimited()
            )
            .unwrap()
    );
    objects
        .weak_map_set_symbol(&map, &key, Value::Object(value.clone()), &mut unlimited())
        .unwrap();
    drop(key);
    (map, value, weak)
}

fn closed_cycle() -> (Objects, Handle, Handle, WeakJsSymbol) {
    let mut objects = Objects::with_limits(None, None);
    let (map, value, weak) = install_closed_cycle(&mut objects);
    (objects, map, value, weak)
}

#[test]
fn symbol_storage_keeps_identity_undefined_and_dense_indices_separate_from_object_keys() {
    let mut objects = Objects::with_limits(None, None);
    let map = empty_map(&mut objects);
    let object_key = objects.create(None).unwrap();
    let keys: Vec<_> = (0..3)
        .map(|_| JsSymbol::new(Some(JsString::from("same"))))
        .collect();
    objects
        .weak_map_set(&map, &object_key, Value::Number(90.0), &mut unlimited())
        .unwrap();
    for (index, key) in keys.iter().enumerate() {
        objects
            .weak_map_set_symbol(&map, key, Value::Number(index as f64), &mut unlimited())
            .unwrap();
    }
    objects
        .weak_map_set_symbol(&map, &keys[0], Value::Undefined, &mut unlimited())
        .unwrap();
    assert!(
        objects
            .weak_map_has_symbol(&map, &keys[0], &mut unlimited())
            .unwrap()
    );
    assert_eq!(
        objects
            .weak_map_get_symbol(&map, &keys[0], &mut unlimited())
            .unwrap(),
        Some(Value::Undefined)
    );
    assert!(
        objects
            .weak_map_delete_symbol(&map, &keys[1], &mut unlimited())
            .unwrap()
    );
    assert!(
        !objects
            .weak_map_delete_symbol(&map, &keys[1], &mut unlimited())
            .unwrap()
    );
    assert_eq!(
        objects
            .weak_map_get_symbol(&map, &keys[2], &mut unlimited())
            .unwrap(),
        Some(Value::Number(2.0))
    );
    assert_eq!(
        objects
            .weak_map_get_symbol(&map, &keys[1], &mut unlimited())
            .unwrap(),
        None
    );
    let unrelated = JsSymbol::new(Some(JsString::from("same")));
    assert!(
        !objects
            .weak_map_has_symbol(&map, &unrelated, &mut unlimited())
            .unwrap()
    );
    assert_eq!(
        objects
            .weak_map_get(&map, &object_key, &mut unlimited())
            .unwrap(),
        Some(Value::Number(90.0))
    );
    assert_eq!(symbol_entries(&objects, &map), 2);
    assert_eq!(object_entries(&objects, &map), 1);
}

#[test]
fn a_host_key_with_zero_local_strong_cells_activates_its_conditional_value() {
    let mut objects = Objects::with_limits(None, None);
    let map = empty_map(&mut objects);
    let value = objects.create(None).unwrap();
    let key = JsSymbol::new(None);
    let weak = key.downgrade();
    objects
        .weak_map_set_symbol(&map, &key, Value::Object(value.clone()), &mut unlimited())
        .unwrap();
    objects.collect([&map], usize::MAX).unwrap();
    assert!(objects.inspect(&value).is_ok());
    assert!(
        objects
            .weak_map_has_symbol(&map, &key, &mut unlimited())
            .unwrap()
    );
    drop(key);
    assert_eq!(weak.upgrade(), None);
    objects.collect([&map], usize::MAX).unwrap();
    stale(&objects, &value);
    assert_eq!(symbol_entries(&objects, &map), 0);
}

#[test]
fn conditional_self_and_object_back_edges_do_not_resurrect_their_own_key() {
    for object_back_edge in [false, true] {
        let mut objects = Objects::with_limits(None, None);
        let map = empty_map(&mut objects);
        let key = JsSymbol::new(None);
        let weak = key.downgrade();
        let value = object_back_edge.then(|| objects.create(None).unwrap());
        let stored = if let Some(value) = &value {
            assert!(
                objects
                    .define(
                        value,
                        "key",
                        data(Value::Symbol(key.clone())),
                        &mut unlimited()
                    )
                    .unwrap()
            );
            Value::Object(value.clone())
        } else {
            Value::Symbol(key.clone())
        };
        objects
            .weak_map_set_symbol(&map, &key, stored, &mut unlimited())
            .unwrap();
        drop(key);
        assert!(weak.upgrade().is_some());
        objects.collect([&map], usize::MAX).unwrap();
        assert_eq!(weak.upgrade(), None);
        assert_eq!(symbol_entries(&objects, &map), 0);
        if let Some(value) = &value {
            stale(&objects, value);
        }
    }
}

#[test]
fn a_host_anchor_activates_a_two_symbol_cycle_and_dropping_it_reclaims_both() {
    let mut objects = Objects::with_limits(None, None);
    let first_map = empty_map(&mut objects);
    let second_map = empty_map(&mut objects);
    let first = JsSymbol::new(None);
    let second = JsSymbol::new(None);
    let first_weak = first.downgrade();
    let second_weak = second.downgrade();
    objects
        .weak_map_set_symbol(
            &first_map,
            &first,
            Value::Symbol(second.clone()),
            &mut unlimited(),
        )
        .unwrap();
    objects
        .weak_map_set_symbol(
            &second_map,
            &second,
            Value::Symbol(first.clone()),
            &mut unlimited(),
        )
        .unwrap();
    drop(second);
    objects
        .collect([&first_map, &second_map], usize::MAX)
        .unwrap();
    assert!(first_weak.upgrade().is_some());
    assert!(second_weak.upgrade().is_some());
    drop(first);
    objects
        .collect([&first_map, &second_map], usize::MAX)
        .unwrap();
    assert_eq!(first_weak.upgrade(), None);
    assert_eq!(second_weak.upgrade(), None);
    assert_eq!(symbol_entries(&objects, &first_map), 0);
    assert_eq!(symbol_entries(&objects, &second_map), 0);
}

#[test]
fn reverse_mixed_object_symbol_chains_reach_a_fixpoint_in_either_allocation_order() {
    for reverse in [false, true] {
        let mut objects = Objects::with_limits(None, None);
        let mut maps: Vec<_> = (0..3).map(|_| empty_map(&mut objects)).collect();
        if reverse {
            maps.reverse();
        }
        let anchor = objects.create(None).unwrap();
        let value = objects.create(None).unwrap();
        let first = JsSymbol::new(None);
        let second = JsSymbol::new(None);
        let first_weak = first.downgrade();
        let second_weak = second.downgrade();
        objects
            .weak_map_set_symbol(
                &maps[0],
                &second,
                Value::Object(value.clone()),
                &mut unlimited(),
            )
            .unwrap();
        objects
            .weak_map_set_symbol(
                &maps[1],
                &first,
                Value::Symbol(second.clone()),
                &mut unlimited(),
            )
            .unwrap();
        objects
            .weak_map_set(
                &maps[2],
                &anchor,
                Value::Symbol(first.clone()),
                &mut unlimited(),
            )
            .unwrap();
        drop(first);
        drop(second);
        objects
            .collect(maps.iter().chain([&anchor]), usize::MAX)
            .unwrap();
        assert!(first_weak.upgrade().is_some());
        assert!(second_weak.upgrade().is_some());
        assert!(objects.inspect(&value).is_ok());
        objects.collect(maps.iter(), usize::MAX).unwrap();
        assert_eq!(first_weak.upgrade(), None);
        assert_eq!(second_weak.upgrade(), None);
        stale(&objects, &anchor);
        stale(&objects, &value);
        assert_eq!(symbol_entries(&objects, &maps[0]), 0);
        assert_eq!(symbol_entries(&objects, &maps[1]), 0);
        assert_eq!(object_entries(&objects, &maps[2]), 0);
    }
}

#[test]
fn an_object_symbol_back_cycle_requires_an_independent_object_root() {
    let mut objects = Objects::with_limits(None, None);
    let symbol_map = empty_map(&mut objects);
    let object_map = empty_map(&mut objects);
    let value = objects.create(None).unwrap();
    let key = JsSymbol::new(None);
    let weak = key.downgrade();
    objects
        .weak_map_set_symbol(
            &symbol_map,
            &key,
            Value::Object(value.clone()),
            &mut unlimited(),
        )
        .unwrap();
    objects
        .weak_map_set(
            &object_map,
            &value,
            Value::Symbol(key.clone()),
            &mut unlimited(),
        )
        .unwrap();
    drop(key);
    objects
        .collect([&symbol_map, &object_map, &value], usize::MAX)
        .unwrap();
    assert!(weak.upgrade().is_some());
    objects
        .collect([&symbol_map, &object_map], usize::MAX)
        .unwrap();
    assert_eq!(weak.upgrade(), None);
    stale(&objects, &value);
    assert_eq!(symbol_entries(&objects, &symbol_map), 0);
    assert_eq!(object_entries(&objects, &object_map), 0);
}

#[test]
fn each_strong_holding_location_marks_when_live_and_is_counted_when_unreachable() {
    let owners = [
        ("property value", "({value:key})"),
        ("property key", "({[key]:0})"),
        ("Symbol wrapper", "Object(key)"),
        (
            "environment",
            "(function(value){return function(){return value;};})(key)",
        ),
        ("bound this and argument", "(function(){}).bind(key,key)"),
        ("Map key and its index", "new Map([[key,0]])"),
        ("Map value", "new Map([[0,key]])"),
        ("Map key/value/index copies", "new Map([[key,key]])"),
        ("Set value and its index", "new Set([key])"),
        (
            "private field",
            "new (class {#value=key;read(){return this.#value;}})()",
        ),
        ("iterator next copies", "Iterator.from({next:key})"),
        (
            "async labelled loop value across suspension",
            "(()=>{let gate=new Promise(()=>{}),step=0,f=async()=>{label:while(step++===0?true:await gate){key}};f();return gate})()",
        ),
        (
            "async super construction earlier argument",
            "(()=>{let gate=new Promise(()=>{});class B{}class C extends B{constructor(){return(async()=>super(key,await gate))()}}new C();return gate})()",
        ),
        (
            "async super computed-key saved this",
            "(()=>{let gate=new Promise(()=>{}),object={async f(){'use strict';return super[await gate]}};object.f.call(key);return gate})()",
        ),
        (
            "async optional key base",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>key?.[await gate];f();return gate})()",
        ),
        (
            "async optional call earlier argument",
            "(()=>{let gate=new Promise(()=>{}),target=()=>{},f=async()=>target?.(key,await gate);f();return gate})()",
        ),
        (
            "async optional call receiver",
            "(()=>{let gate=new Promise(()=>{});Symbol.prototype.pendingOptional=function(){};let f=async()=>key.pendingOptional?.(await gate);f();delete Symbol.prototype.pendingOptional;return gate})()",
        ),
        (
            "async partial object earlier value",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>({saved:key,later:await gate});f();return gate})()",
        ),
        (
            "async object converted key",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>({[key]:await gate});f();return gate})()",
        ),
        (
            "async boxed with binding object",
            "(()=>{let gate=new Promise(()=>{});(async()=>{with(key){await gate}})();return gate})()",
        ),
        (
            "async for-await cached next callable",
            "(()=>{let gate=new Promise(()=>{}),next=(()=>{let value=key;return()=>{value;return gate}})(),iterator={next},source={[Symbol.asyncIterator](){return iterator}};(async()=>{for await(const x of source){}})();iterator.next=null;next=null;return gate})()",
        ),
        (
            "async for-await loop value",
            "(()=>{let gate=new Promise(()=>{}),n=0,source={[Symbol.asyncIterator](){return this},next(){return n++===0?{done:false,value:0}:gate}};(async()=>{for await(const x of source){key}})();return gate})()",
        ),
        (
            "async for-await pending return",
            "(()=>{let gate=new Promise(()=>{}),source={[Symbol.asyncIterator](){return this},next(){return{done:false,value:0}},return(){return gate}};(async()=>{for await(const x of source){return key}})();return gate})()",
        ),
        (
            "async for-await pending throw",
            "(()=>{let gate=new Promise(()=>{}),source={[Symbol.asyncIterator](){return this},next(){return{done:false,value:0}},return(){return gate}};(async()=>{for await(const x of source){throw key}})();return gate})()",
        ),
        (
            "async iteration input across awaited target",
            "(()=>{let gate=new Promise(()=>{}),values=[key],target={};(async()=>{for(target[await gate] of values){}})();values[0]=null;return gate})()",
        ),
        (
            "async array binding cached next",
            "(()=>{let gate=new Promise(()=>{}),next=(()=>{let value=key;return()=>({done:false,value:(value,undefined)})})(),iterator={next,return(){return{}}};(async()=>{let[x=await gate]={[Symbol.iterator](){return iterator}}})();iterator.next=null;next=null;return gate})()",
        ),
        (
            "async array binding parent cached next",
            "(()=>{let gate=new Promise(()=>{}),next=(()=>{let value=key;return()=>({done:false,value:(value,[])})})(),iterator={next,return(){return{}}};(async()=>{let[[x=await gate]]={[Symbol.iterator](){return iterator}}})();iterator.next=null;next=null;return gate})()",
        ),
        (
            "async iteration array binding cached next",
            "(()=>{let gate=new Promise(()=>{}),next=(()=>{let value=key;return()=>({done:false,value:(value,undefined)})})(),inner={next,[Symbol.iterator](){return this},return(){return{}}},source=[inner];(async()=>{for(const[x=await gate]of source){}})();inner.next=null;next=null;source[0]=null;return gate})()",
        ),
        (
            "async var array cached lexical reference",
            "(()=>{let gate=new Promise(()=>{});(async function(x){var[x=await gate]=[]})(key);return gate})()",
        ),
        (
            "async var array cached object reference",
            "(()=>{let gate=new Promise(()=>{}),object={x:key};(async()=>{with(object){var[x=await gate]=[]}})();return gate})()",
        ),
        (
            "generator yielded parameter",
            "(()=>{let iterator=(function*(x){yield 0;return x})(key);iterator.next();return iterator})()",
        ),
        (
            "generator yielded strict receiver",
            "(()=>{let iterator=(function*(){'use strict';yield 0;return this}).call(key);iterator.next();return iterator})()",
        ),
        (
            "generator yielded callee property",
            "(()=>{let f=function*(arguments){yield 0};f.kept=key;let iterator=f(0);f=null;iterator.next();return iterator})()",
        ),
        (
            "generator yielded binary left operand",
            "(()=>{let iterator=(function*(){return key+(yield 0)})();iterator.next();return iterator})()",
        ),
        (
            "generator yielded pending return",
            "(()=>{let iterator=(function*(){try{yield 0}finally{yield 1}})();iterator.next();iterator.return(key);return iterator})()",
        ),
        (
            "generator yielded pending throw",
            "(()=>{let iterator=(function*(){try{yield 0}finally{yield 1}})();iterator.next();iterator.throw(key);return iterator})()",
        ),
        (
            "generator yielded binding cached next",
            "(()=>{let next=(()=>{let value=key;return()=>({done:false,value:(value,undefined)})})(),source={next,[Symbol.iterator](){return this},return(){return{}}},iterator=(function*(){let[x=yield 0]=source})();iterator.next();source.next=null;next=null;return iterator})()",
        ),
        (
            "generator yielded object key",
            "(()=>{let iterator=(function*(){return{[key]:yield 0}})();iterator.next();return iterator})()",
        ),
        (
            "generator yielded assignment raw key",
            "(()=>{let iterator=(function*(){({})[key]=yield 0})();iterator.next();return iterator})()",
        ),
        (
            "generator yielded earlier call argument",
            "(()=>{let call=()=>{},iterator=(function*(){return call(key,yield 0)})();iterator.next();return iterator})()",
        ),
        (
            "generator suspended parameter",
            "(function*(x){return x})(key)",
        ),
        (
            "generator suspended strict receiver",
            "(function*(){'use strict';return this}).call(key)",
        ),
        (
            "generator suspended callee property",
            "(()=>{let f=function*(arguments){};f.kept=key;let result=f(0);f=null;return result})()",
        ),
        (
            "generator suspended private receiver",
            "(new(class{#value=key;*g(){return this.#value}})).g()",
        ),
        (
            "async catch array cached next",
            "(()=>{let gate=new Promise(()=>{}),next=(()=>{let value=key;return()=>({done:false,value:(value,undefined)})})(),iterator={next,return(){return{}}};(async()=>{try{throw{[Symbol.iterator](){return iterator}}}catch([x=await gate]){}})();iterator.next=null;next=null;return gate})()",
        ),
        (
            "async catch array initialized binding",
            "(()=>{let gate=new Promise(()=>{}),input=[key];(async()=>{try{throw input}catch([x,y=await gate]){}})();input[0]=null;return gate})()",
        ),
        (
            "async object binding primitive source",
            "(()=>{let gate=new Promise(()=>{});(async()=>{let{[await gate]:x}=key})();return gate})()",
        ),
        (
            "async object binding pending converted key",
            "(()=>{let gate=new Promise(()=>{}),source={[key]:undefined};(async()=>{let{[key]:x=await gate}=source})();delete source[key];return gate})()",
        ),
        (
            "async object binding excluded symbol key",
            "(()=>{let gate=new Promise(()=>{}),source={[key]:1};(async()=>{let{[key]:x,y=await gate,...rest}=source})();delete source[key];return gate})()",
        ),
        (
            "async object binding repeated physical key slots",
            "(()=>{let gate=new Promise(()=>{}),source={[key]:undefined};(async()=>{let{[key]:x=1,[key]:y=await gate,...rest}=source})();delete source[key];return gate})()",
        ),
        (
            "async switch input",
            "(()=>{let gate=new Promise(()=>{});(async()=>{switch(key){case await gate:break}})();return gate})()",
        ),
        (
            "async switch statement value",
            "(()=>{let gate=new Promise(()=>{});(async()=>{switch(0){case 0:key;await gate}})();return gate})()",
        ),
        (
            "async object repeated physical symbol key slots",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>({[key]:1,[key]:await gate});f();return gate})()",
        ),
        (
            "async tagged template earlier raw argument",
            "(()=>{let gate=new Promise(()=>{}),tag=()=>{},f=async()=>tag`${key}${await gate}`;f();return gate})()",
        ),
        (
            "async tagged template receiver",
            "(()=>{let gate=new Promise(()=>{});Symbol.prototype.pendingTag=function(){};let f=async()=>key.pendingTag`${await gate}`;f();delete Symbol.prototype.pendingTag;return gate})()",
        ),
        (
            "async assignment raw reference base",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{key.x=await gate};f();return gate})()",
        ),
        (
            "async assignment super receiver",
            "(()=>{let gate=new Promise(()=>{});class B{set x(value){}}class C extends B{async f(){super.x=await gate}}C.prototype.f.call(key);return gate})()",
        ),
        (
            "async assignment raw reference key",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{({})[key]=await gate};f();return gate})()",
        ),
        (
            "async compound assignment raw left value",
            "(()=>{let gate=new Promise(()=>{}),object={x:key},f=async()=>{object.x+=await gate};f();object.x=0;return gate})()",
        ),
        (
            "async for loop value across awaited update",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{let n=0;for(;n++===0;await gate){key}};f();return gate})()",
        ),
        (
            "async for-in loop value across suspension",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{for(const x in {a:1,b:2}){if(x==='a'){key}else await gate}};f();return gate})()",
        ),
        (
            "async for-of loop value across suspension",
            "(()=>{let gate=new Promise(()=>{}),n=0,source={[Symbol.iterator](){return this},next(){return{done:++n>2,value:n}}},f=async()=>{for(const x of source){if(x===1){key}else await gate}};f();return gate})()",
        ),
        (
            "async cached for-of next callable environment",
            "(()=>{let gate=new Promise(()=>{}),cached=(()=>{let value=key;return function(){return{done:false,value}}})(),iterator={next:cached},source={[Symbol.iterator](){return iterator}},f=async()=>{for(const value of source){await gate;break}};f();iterator.next=null;cached=null;return gate})()",
        ),
        (
            "async loop value across awaited condition",
            "(()=>{let gate=new Promise(()=>{}),step=0,f=async()=>{while(step++===0?true:await gate){key}};f();return gate})()",
        ),
        (
            "async protected return across awaited finally",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{try{return key}finally{await gate}};f();return gate})()",
        ),
        (
            "async protected throw across awaited finally",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{try{throw key}finally{await gate}};f();return gate})()",
        ),
        (
            "async protected normal completion across awaited finally",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{try{key}finally{await gate}};f();return gate})()",
        ),
        (
            "async root and protected return completion copies",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{key;try{return key}finally{await gate}};f();return gate})()",
        ),
        (
            "async root statement completion across suspension",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{key;const ignored=await gate};f();return gate})()",
        ),
        (
            "async block statement completion across suspension",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{{key;const ignored=await gate}};f();return gate})()",
        ),
        (
            "async root and block statement completion copies",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{key;{key;const ignored=await gate}};f();return gate})()",
        ),
        (
            "async selected branch environment across suspension",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{if(true){const value=key;await gate;return value}};f();return gate})()",
        ),
        (
            "async nested block environment across suspension",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{{const value=key;await gate;return value}};f();return gate})()",
        ),
        (
            "async constructor target across awaited argument",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>new key(await gate);f();return gate})()",
        ),
        (
            "async member base across awaited key",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>key[await gate];f();return gate})()",
        ),
        (
            "async partial array across suspension",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>[key,await gate];f();return gate})()",
        ),
        (
            "async call earlier argument across suspension",
            "(()=>{let gate=new Promise(()=>{}),target=()=>{},f=async()=>target(key,await gate);f();return gate})()",
        ),
        (
            "async call receiver across suspension",
            "(()=>{let gate=new Promise(()=>{});Symbol.prototype.pendingMethod=function(){};let f=async()=>key.pendingMethod(await gate);f();delete Symbol.prototype.pendingMethod;return gate})()",
        ),
        (
            "async noncallable target across suspension",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>key(await gate);f();return gate})()",
        ),
        (
            "async binary left operand across suspension",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{return key+await gate};f();return gate})()",
        ),
        (
            "source awaited var across suspension",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{var value=await key;await gate;return value};f();return gate})()",
        ),
        (
            "source awaited lexical across suspension",
            "(()=>{let gate=new Promise(()=>{}),f=async()=>{const value=await key;await gate;return value};f();return gate})()",
        ),
        (
            "source async parameter across suspension",
            "(()=>{let gate=new Promise(()=>{}),f=async value=>{await gate;return value};f(key);return gate})()",
        ),
        (
            "Array.fromAsync mapper this across suspension",
            "(()=>{let gate=new Promise(()=>{});Array.fromAsync({0:gate,length:1},x=>x,key);return gate})()",
        ),
        (
            "Array.fromAsync close error across suspension",
            "(()=>{let gate=new Promise(()=>{}),source={[Symbol.asyncIterator](){return this},next(){return{value:1,done:false}},return(){return gate}};Array.fromAsync(source,()=>{throw key});return gate})()",
        ),
        (
            "async disposal resource across suspension",
            "(()=>{let gate=new Promise(()=>{}),stack=new AsyncDisposableStack;stack.adopt(key,()=>{});stack.defer(()=>gate);stack.disposeAsync();return gate})()",
        ),
        (
            "async disposal error across suspension",
            "(()=>{let gate=new Promise(()=>{}),stack=new AsyncDisposableStack;stack.defer(()=>gate);stack.defer(()=>{throw key});stack.disposeAsync();return gate})()",
        ),
    ];
    let mut realm = Realm::default();
    for (name, expression) in owners {
        let key = symbol(
            realm
                .eval(&format!("var key=Symbol('census'),holder={expression};key"))
                .unwrap(),
        );
        if matches!(
            name,
            "Array.fromAsync close error across suspension"
                | "source awaited lexical across suspension"
                | "source awaited var across suspension"
                | "async for-await cached next callable"
                | "async for-await loop value"
                | "async for-await pending return"
                | "async for-await pending throw"
        ) {
            loop {
                match realm.run_next_promise_job().unwrap() {
                    crate::PromiseJobOutcome::NoJob => break,
                    crate::PromiseJobOutcome::Completed => (),
                    crate::PromiseJobOutcome::Reported(error) => panic!("{error}"),
                }
            }
        }
        let weak = key.downgrade();
        let map = object(realm.eval("new WeakMap()").unwrap());
        let _map_root = realm
            .root_value(Value::Object(map.clone()), usize::MAX)
            .unwrap();
        let value = object(realm.eval("({})").unwrap());
        realm
            .objects
            .weak_map_set_symbol(&map, &key, Value::Object(value.clone()), &mut unlimited())
            .unwrap();
        drop(key);
        realm.eval("key=null").unwrap();
        realm.collect(usize::MAX).unwrap();
        assert!(weak.upgrade().is_some(), "live {name}");
        assert!(
            realm.objects.inspect(&value).is_ok(),
            "conditional value for live {name}"
        );
        realm.eval("holder=null").unwrap();
        realm.collect(usize::MAX).unwrap();
        assert_eq!(
            weak.upgrade(),
            None,
            "unreachable {name} must not look like an external Arc owner"
        );
        stale(&realm.objects, &value);
        assert_eq!(symbol_entries(&realm.objects, &map), 0, "{name}");
    }
}

#[test]
fn an_independent_thread_strong_owner_needs_no_heap_root_registration() {
    let mut objects = Objects::with_limits(None, None);
    let map = empty_map(&mut objects);
    let value = objects.create(None).unwrap();
    let key = JsSymbol::new(None);
    let weak = key.downgrade();
    objects
        .weak_map_set_symbol(&map, &key, Value::Object(value.clone()), &mut unlimited())
        .unwrap();
    let (ready_tx, ready_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let owner = thread::spawn(move || {
        ready_tx.send(()).unwrap();
        release_rx.recv().unwrap();
        drop(key);
    });
    ready_rx.recv().unwrap();
    let collected = objects.collect([&map], usize::MAX);
    let identity_live = weak.upgrade().is_some();
    let value_live = objects.inspect(&value).is_ok();
    release_tx.send(()).unwrap();
    owner.join().unwrap();
    collected.unwrap();
    assert!(identity_live);
    assert!(value_live);
    assert_eq!(weak.upgrade(), None);
    objects.collect([&map], usize::MAX).unwrap();
    stale(&objects, &value);
}

#[test]
fn a_pending_private_assignment_counts_its_internal_name_without_creating_an_external_owner() {
    let mut realm = Realm::default();
    realm.eval("let map=new WeakMap(),holder=(()=>{let gate=new Promise(()=>{});class C{#x=0;async f(){this.#x=await gate}}const instance=new C();instance.f();return{gate,instance}})()").unwrap();
    let map = object(realm.eval("map").unwrap());
    let instance = object(realm.eval("holder.instance").unwrap());
    let key = realm
        .objects
        .inspect(&instance)
        .unwrap()
        .private_elements
        .first()
        .unwrap()
        .0
        .symbol()
        .clone();
    let weak = key.downgrade();
    let value = realm.objects.create(None).unwrap();
    realm
        .objects
        .weak_map_set_symbol(&map, &key, Value::Object(value.clone()), &mut unlimited())
        .unwrap();
    drop(key);
    drop(instance);
    realm.collect(usize::MAX).unwrap();
    assert!(weak.upgrade().is_some());
    assert!(realm.objects.inspect(&value).is_ok());
    realm.eval("holder=null").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(weak.upgrade(), None);
    stale(&realm.objects, &value);
    assert_eq!(symbol_entries(&realm.objects, &map), 0);
}

#[test]
fn another_realms_strong_symbol_cell_is_external_to_the_collecting_heap() {
    let mut objects = Objects::with_limits(None, None);
    let map = empty_map(&mut objects);
    let value = objects.create(None).unwrap();
    let key = JsSymbol::new(None);
    let weak = key.downgrade();
    objects
        .weak_map_set_symbol(&map, &key, Value::Object(value.clone()), &mut unlimited())
        .unwrap();
    let mut other = Realm::default();
    let holder = object(other.eval("({})").unwrap());
    assert!(
        other
            .objects
            .define(
                &holder,
                "symbol",
                data(Value::Symbol(key.clone())),
                &mut unlimited()
            )
            .unwrap()
    );
    let other_root = other.root_value(Value::Object(holder), usize::MAX).unwrap();
    drop(key);
    objects.collect([&map], usize::MAX).unwrap();
    assert!(weak.upgrade().is_some());
    assert!(objects.inspect(&value).is_ok());
    drop(other_root);
    drop(other);
    assert_eq!(weak.upgrade(), None);
    objects.collect([&map], usize::MAX).unwrap();
    stale(&objects, &value);
}

struct UpgradingRoots<'a> {
    map: &'a Handle,
    key: WeakJsSymbol,
    promoted: Rc<RefCell<Vec<JsSymbol>>>,
    events: Rc<Cell<u8>>,
    yielded: bool,
    promote_on_next: bool,
}

impl<'a> Iterator for UpgradingRoots<'a> {
    type Item = &'a Handle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.yielded {
            return None;
        }
        self.yielded = true;
        if self.promote_on_next {
            if let Some(key) = self.key.upgrade() {
                self.promoted.borrow_mut().push(key);
            } else {
                self.events.set(self.events.get() | 4);
            }
        }
        self.events.set(self.events.get() | 1);
        Some(self.map)
    }
}

impl Drop for UpgradingRoots<'_> {
    fn drop(&mut self) {
        if let Some(key) = self.key.upgrade() {
            self.promoted.borrow_mut().push(key);
        } else {
            self.events.set(self.events.get() | 4);
        }
        self.events.set(self.events.get() | 2);
    }
}

#[test]
fn caller_root_iteration_and_destruction_upgrade_before_any_logical_death() {
    for promote_on_next in [false, true] {
        let (mut objects, map, value, weak) = closed_cycle();
        let promoted = Rc::new(RefCell::new(Vec::new()));
        let events = Rc::new(Cell::new(0));
        let roots = UpgradingRoots {
            map: &map,
            key: weak.clone(),
            promoted: promoted.clone(),
            events: events.clone(),
            yielded: false,
            promote_on_next,
        };
        objects.collect(roots, usize::MAX).unwrap();
        assert_eq!(events.get(), 3);
        assert_eq!(promoted.borrow().len(), if promote_on_next { 2 } else { 1 });
        assert!(weak.upgrade().is_some());
        assert!(objects.inspect(&value).is_ok());
        promoted.borrow_mut().clear();
        objects.collect([&map], usize::MAX).unwrap();
        assert_eq!(weak.upgrade(), None);
        stale(&objects, &value);
    }
}

#[test]
fn a_late_host_promotion_activates_earlier_candidates_before_any_retirement() {
    let mut objects = Objects::with_limits(None, None);
    let early_map = empty_map(&mut objects);
    let late_map = empty_map(&mut objects);
    let value = objects.create(None).unwrap();
    let early = JsSymbol::new(None);
    let late = JsSymbol::new(None);
    let early_weak = early.downgrade();
    let late_weak = late.downgrade();
    objects
        .weak_map_set_symbol(
            &early_map,
            &early,
            Value::Object(value.clone()),
            &mut unlimited(),
        )
        .unwrap();
    objects
        .weak_map_set_symbol(
            &late_map,
            &late,
            Value::Symbol(early.clone()),
            &mut unlimited(),
        )
        .unwrap();
    assert!(
        objects
            .define(
                &value,
                "back",
                data(Value::Symbol(late.clone())),
                &mut unlimited()
            )
            .unwrap()
    );
    drop(early);
    drop(late);
    let promoted = Rc::new(RefCell::new(None));
    let sink = promoted.clone();
    let key = late_weak.clone();
    let result = Objects::with_symbol_gc_test_hooks(
        Some(Box::new(move || {
            *sink.borrow_mut() = key.upgrade();
        })),
        None,
        || objects.collect([&early_map, &late_map], usize::MAX),
    );
    result.unwrap();
    assert!(promoted.borrow().is_some());
    assert!(early_weak.upgrade().is_some());
    assert!(late_weak.upgrade().is_some());
    assert!(objects.inspect(&value).is_ok());
    // The earlier candidate is reached only through the newly external late
    // candidate. A root-discovery loop that retires as it scans is invalid.
    drop(promoted.borrow_mut().take());
    objects
        .collect([&early_map, &late_map], usize::MAX)
        .unwrap();
    assert_eq!(early_weak.upgrade(), None);
    assert_eq!(late_weak.upgrade(), None);
    stale(&objects, &value);
}

#[test]
fn logical_retirement_blocks_weak_resurrection_before_physical_cleanup() {
    let (mut objects, map, value, weak) = closed_cycle();
    let observed = Rc::new(Cell::new(None));
    let sink = observed.clone();
    let key = weak.clone();
    let result = Objects::with_symbol_gc_test_hooks(
        None,
        Some(Box::new(move || {
            sink.set(Some(key.upgrade().is_none()));
        })),
        || objects.collect([&map], usize::MAX),
    );
    result.unwrap();
    // The callback ran after unlock but before the value/back-edge and collector
    // pin were released, so a physical Arc existed and logical death decided it.
    assert_eq!(observed.get(), Some(true));
    assert_eq!(weak.upgrade(), None);
    stale(&objects, &value);
    assert_eq!(symbol_entries(&objects, &map), 0);
}

#[test]
fn failing_work_prefixes_preserve_entries_handles_and_symbol_liveness() {
    let (mut calibration, map, _, _) = closed_cycle();
    let complete = calibration.collect([&map], usize::MAX).unwrap().work_used;
    assert!(complete > 2);
    // Derive a late failing prefix from the same graph, without hardcoding a
    // success allowance or changing any default quota.
    for limit in [0, 1, complete / 2, complete - 1] {
        let (mut objects, map, value, weak) = closed_cycle();
        assert_eq!(
            objects.collect([&map], limit),
            Err(spite_heap::Error::Limit)
        );
        assert!(weak.upgrade().is_some());
        assert!(objects.inspect(&value).is_ok());
        assert_eq!(symbol_entries(&objects, &map), 1);
        objects.collect([&map], usize::MAX).unwrap();
        assert_eq!(weak.upgrade(), None);
        stale(&objects, &value);
    }
}

#[test]
fn an_index_invariant_error_is_reported_before_any_symbol_is_retired() {
    let (mut objects, map, value, weak) = closed_cycle();
    objects
        .object_mut(&map)
        .unwrap()
        .weak_map
        .as_mut()
        .unwrap()
        .symbol_index
        .insert(weak.clone(), 7);
    assert_eq!(
        objects.collect([&map], usize::MAX),
        Err(spite_heap::Error::CollectionState)
    );
    assert!(weak.upgrade().is_some());
    assert!(objects.inspect(&value).is_ok());
    let stored = objects.inspect(&map).unwrap().weak_map.as_ref().unwrap();
    assert_eq!(stored.symbol_entries.len(), 1);
    assert_eq!(stored.symbol_index.get(&weak), Some(&7));
    objects
        .object_mut(&map)
        .unwrap()
        .weak_map
        .as_mut()
        .unwrap()
        .symbol_index
        .insert(weak.clone(), 0);
    objects.collect([&map], usize::MAX).unwrap();
    assert_eq!(weak.upgrade(), None);
    stale(&objects, &value);
}

#[test]
fn invalid_caller_roots_and_precommit_unwind_leave_the_graph_usable() {
    let mut objects = Objects::with_limits(None, None);
    let old = objects.create(None).unwrap();
    objects.collect([], usize::MAX).unwrap();
    let (map, value, weak) = install_closed_cycle(&mut objects);
    let mut other = Objects::with_limits(None, None);
    let foreign = other.create(None).unwrap();
    for (bad, expected) in [
        (old, spite_heap::Error::StaleHandle),
        (foreign, spite_heap::Error::ForeignHandle),
    ] {
        assert_eq!(objects.collect([&map, &bad], usize::MAX), Err(expected));
        assert!(weak.upgrade().is_some());
        assert!(objects.inspect(&value).is_ok());
        assert_eq!(symbol_entries(&objects, &map), 1);
    }
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let roots =
            std::iter::once(&map).inspect(|_| panic!("caller iterator failed before commit"));
        let _ = objects.collect(roots, usize::MAX);
    }));
    assert!(unwind.is_err());
    assert!(weak.upgrade().is_some());
    assert!(objects.inspect(&value).is_ok());
    assert_eq!(symbol_entries(&objects, &map), 1);
    objects.collect([&map], usize::MAX).unwrap();
    assert_eq!(weak.upgrade(), None);
    stale(&objects, &value);
}

#[test]
fn sparse_map_and_set_index_scans_precharge_capacity_before_symbol_retirement() {
    for is_set in [false, true] {
        let (mut objects, map, value, weak) = closed_cycle();
        let prototype = objects.create(None).unwrap();
        let key = weak.upgrade().unwrap();
        let holder = if is_set {
            let holder = objects
                .create_set(
                    &prototype,
                    crate::object::SetData::default(),
                    &mut unlimited(),
                )
                .unwrap();
            objects
                .set_insert(&holder, Value::Symbol(key), &mut unlimited())
                .unwrap();
            holder
        } else {
            let holder = objects.create_map(&prototype).unwrap();
            objects
                .map_set(
                    &holder,
                    Value::Symbol(key),
                    Value::Number(1.0),
                    &mut unlimited(),
                )
                .unwrap();
            holder
        };
        // This pass retains the exact same graph. Its actual complete work is
        // more than the ordinary work needed once holder is no longer a root.
        let ordinary = objects
            .collect([&map, &holder], usize::MAX)
            .unwrap()
            .work_used;
        assert!(ordinary > 1);
        let requested = ordinary.checked_mul(4).unwrap();
        let capacity = {
            let record = objects.object_mut(&holder).unwrap();
            // CollectionKey hashes immutable values/identities, never Symbol liveness.
            #[allow(clippy::mutable_key_type)]
            let index = if is_set {
                &mut record.set.as_mut().unwrap().index
            } else {
                &mut record.map.as_mut().unwrap().index
            };
            assert_eq!(index.len(), 1);
            index.try_reserve(requested).unwrap();
            assert!(index.capacity() > ordinary);
            index.capacity()
        };
        // Only storage capacity changed. The same nonzero graph-derived budget
        // must now fail before the sparse bucket scan, not after logical death.
        assert_eq!(
            objects.collect([&map], ordinary),
            Err(spite_heap::Error::Limit)
        );
        assert!(weak.upgrade().is_some());
        assert!(objects.inspect(&value).is_ok());
        assert!(objects.inspect(&holder).is_ok());
        assert_eq!(symbol_entries(&objects, &map), 1);
        let record = objects.inspect(&holder).unwrap();
        // CollectionKey hashes immutable values/identities, never Symbol liveness.
        #[allow(clippy::mutable_key_type)]
        let index = if is_set {
            &record.set.as_ref().unwrap().index
        } else {
            &record.map.as_ref().unwrap().index
        };
        assert_eq!(index.len(), 1);
        assert_eq!(index.capacity(), capacity);
        objects.collect([&map], usize::MAX).unwrap();
        assert_eq!(weak.upgrade(), None);
        stale(&objects, &value);
        stale(&objects, &holder);
        assert_eq!(symbol_entries(&objects, &map), 0);
    }
}
