//! Script-visible Symbols, global descriptors, and well-known hooks.

use spite_core::WellKnownSymbol;
use spite_runtime::{Realm, Value};

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}

#[test]
fn symbol_global_is_mutable_configurable_and_preserves_the_intrinsic() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        "let d=Object.getOwnPropertyDescriptor(globalThis,'Symbol'),S=Symbol,s=S('x');d.value===Symbol && d.writable && !d.enumerable && d.configurable && Symbol.length===0 && Symbol.name==='Symbol'",
    );
    check(
        &mut realm,
        "globalThis.Symbol=7;Symbol===7 && s.constructor===S && (delete globalThis.Symbol) && typeof Symbol==='undefined' && globalThis.Symbol===undefined && s.constructor===S",
    );
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "s.toString()==='Symbol(x)' && (globalThis.Symbol=S,Symbol('x')!==s) && Symbol.prototype===Object.getPrototypeOf(s)",
    );
}

#[test]
fn fresh_and_registered_symbols_preserve_identity_across_script_evaluations_and_realms() {
    let mut first = Realm::default();
    let mut second = Realm::default();
    let fresh = first.eval("Symbol('same')").unwrap();
    assert_ne!(fresh, second.eval("Symbol('same')").unwrap());
    let registered = first.eval("Symbol.for('script-registry')").unwrap();
    first.collect(usize::MAX).unwrap();
    assert_eq!(
        registered,
        first.eval("Symbol.for('script-registry')").unwrap()
    );
    assert_eq!(
        registered,
        second.eval("Symbol.for('script-registry')").unwrap()
    );
    for symbol in WellKnownSymbol::ALL {
        let source = format!("Symbol.{}", symbol.name());
        assert_eq!(first.eval(&source), Ok(Value::Symbol(symbol.symbol())));
        assert_eq!(first.eval(&source), second.eval(&source));
    }
}

// 6.1.5.1 and Symbol.asyncDispose / Symbol.dispose (20.4.2).
#[test]
fn disposal_symbols_have_standard_descriptions_and_frozen_constructor_properties() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        "for (const name of ['dispose','asyncDispose']) { const value=Symbol[name],d=Object.getOwnPropertyDescriptor(Symbol,name); if(typeof value!=='symbol'||value.description!=='Symbol.'+name||d.value!==value||d.writable||d.enumerable||d.configurable||Symbol.keyFor(value)!==undefined||value===Symbol(value.description)||value===Symbol.for(value.description))throw 1; } Symbol.dispose!==Symbol.asyncDispose",
    );
}

#[test]
fn disposal_symbol_properties_reject_mutation_and_remain_distinct_property_keys() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        "'use strict';const saved=Symbol.dispose;let rejected=0;try{Symbol.dispose=Symbol()}catch(e){if(e instanceof TypeError)rejected++}try{Object.defineProperty(Symbol,'asyncDispose',{value:Symbol()})}catch(e){if(e instanceof TypeError)rejected++}const object={[Symbol.dispose]:'sync',[Symbol.asyncDispose]:'async'},keys=Reflect.ownKeys(object);rejected===2&&!Reflect.deleteProperty(Symbol,'dispose')&&Symbol.dispose===saved&&keys.length===2&&keys[0]===Symbol.dispose&&keys[1]===Symbol.asyncDispose&&object[keys[0]]==='sync'&&object[keys[1]]==='async'&&object['Symbol.dispose']===undefined",
    );
}

#[test]
fn disposal_symbols_remain_valid_weak_targets_across_collection() {
    let mut realm = Realm::default();
    realm
        .eval("let map=new WeakMap([[Symbol.dispose,1],[Symbol.asyncDispose,2]]),set=new WeakSet([Symbol.dispose,Symbol.asyncDispose]),ref=new WeakRef(Symbol.dispose),asyncRef=new WeakRef(Symbol.asyncDispose)")
        .unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    check(
        &mut realm,
        "map.get(Symbol.dispose)===1&&map.get(Symbol.asyncDispose)===2&&set.has(Symbol.dispose)&&set.has(Symbol.asyncDispose)&&ref.deref()===Symbol.dispose&&asyncRef.deref()===Symbol.asyncDispose",
    );
}

#[test]
fn computed_keys_and_coercion_hooks_are_usable_from_scripts() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        "let a=Symbol('x'),b=Symbol('x'),o={[a]:1,[b]:2},copy=Object.assign({},o);o[a]===1 && o[b]===2 && o.x===undefined && copy[a]===1 && copy[b]===2 && typeof a==='symbol' && Boolean(a)",
    );
    check(
        &mut realm,
        "let hint='',k={[Symbol.toPrimitive]:(h)=>{hint=h;return a;}};o[k]===1 && hint==='string'",
    );
    check(
        &mut realm,
        "let matcher={[Symbol.match]:true},caught=false;try{'x'.includes(matcher);}catch(e){caught=e instanceof TypeError;}caught",
    );
    check(
        &mut realm,
        "let target={[Symbol.hasInstance]:(v)=>v===7};7 instanceof target && !(8 instanceof target) && Object.prototype.toString.call({[Symbol.toStringTag]:'Custom'})==='[object Custom]'",
    );
}

#[test]
fn script_iterators_use_standard_aliases_and_code_point_iteration() {
    let mut realm = Realm::default();
    check(
        &mut realm,
        "let i=[1,,3][Symbol.iterator]();Array.prototype[Symbol.iterator]===Array.prototype.values && i[Symbol.iterator]()===i && i.next().value===1 && i.next().value===undefined && i.next().value===3 && i.next().done",
    );
    check(
        &mut realm,
        "i='a\\uD83D\\uDE00\\uD800'[Symbol.iterator]();i[Symbol.iterator]()===i && i.next().value==='a' && i.next().value==='\\uD83D\\uDE00' && i.next().value==='\\uD800' && i.next().done",
    );
    check(
        &mut realm,
        "function f(x){let it=arguments[Symbol.iterator]();x=2;return it.next().value===2 && it.next().done;}f(1)",
    );
}
