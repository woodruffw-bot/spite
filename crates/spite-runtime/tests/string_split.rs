//! String split: UTF-16 boundaries, conversion order, and object split hooks.

mod common;

use common::REALM_ENTRIES;
use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn plain_separators_preserve_empty_parts_and_use_nonoverlapping_matches() {
    check(
        "let a=',a,,b,'.split(',');a.length===5 && a.join('|')==='|a||b|' && Object.keys(a).join(',')==='0,1,2,3,4' && 'aaaaa'.split('aa').join('|')==='||a'",
    );
    check(
        "''.split('').length===0 && ''.split('x').length===1 && ''.split('x')[0]==='' && 'abc'.split('abc').join('|')==='|' && 'abc'.split('abcd')[0]==='abc' && 'abc'.split('x')[0]==='abc'",
    );
    check(
        "'undefined'.split().length===1 && 'undefined'.split(undefined)[0]==='undefined' && 'anullb'.split(null).join('|')==='a|b' && 'atrueb'.split(true).join('|')==='a|b' && 'a1b'.split(1n).join('|')==='a|b'",
    );
    check(
        "String.prototype.split.call(1001,0).join('|')==='1||1' && String.prototype.split.call(false,'a').join('|')==='f|lse' && String.prototype.split.call(123n,2).join('|')==='1|3'",
    );
    check(
        "let a='a,b'.split(','),b='a,b'.split(',');a!==b && Array.isArray(a) && Object.getPrototypeOf(a)===Array.prototype",
    );
}

#[test]
fn empty_separators_split_code_units_and_preserve_unpaired_surrogates() {
    check(
        "let a='A\\uD834\\uDF06\\uD800B'.split('');a.length===5 && a[0]==='A' && a[1]==='\\uD834' && a[2]==='\\uDF06' && a[3]==='\\uD800' && a[4]==='B' && a.join('')==='A\\uD834\\uDF06\\uD800B'",
    );
    check(
        "let a='\\uD834\\uDF06\\uD834\\uDF06'.split('\\uDF06');a.length===3 && a[0]==='\\uD834' && a[1]==='\\uD834' && a[2]==='' && 'A\\uD834\\uDF06B'.split('\\uD834\\uDF06').join('|')==='A|B'",
    );
}

#[test]
fn limits_use_uint32_conversion_and_stop_without_appending_the_tail() {
    check(
        "'a,b,c'.split(',',1).join('|')==='a' && 'a,b,c'.split(',',2.9).join('|')==='a|b' && 'abc'.split('',2).join('|')==='a|b' && 'a,b,c'.split(',',-1).length===3 && 'a,b,c'.split(',',4294967297).length===1",
    );
    for limit in [
        "0",
        "-0",
        "null",
        "false",
        "NaN",
        "Infinity",
        "-Infinity",
        "4294967296",
        "0.9",
    ] {
        check(&format!(
            "'a,b'.split(',',{limit}).length===0 && 'abc'.split(undefined,{limit}).length===0"
        ));
    }
    check("'abc'.split(undefined,1)[0]==='abc' && 'abc'.split('',undefined).length===3");
    for limit in ["1n", "Symbol()"] {
        check(&format!(
            "let caught=false;try{{'a,b'.split(',',{limit});}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
}

#[test]
fn object_hook_delegation_preserves_original_arguments_and_returns_any_value() {
    check(
        "let log='',receiver={toString(){throw 7;}},limit={valueOf(){throw 8;}},separator={get [Symbol.split](){log+='g';return function(o,l){'use strict';if(this!==separator || o!==receiver || l!==limit || arguments.length!==2)throw 9;log+='c';return 3;};},toString(){throw 10;}};String.prototype.split.call(receiver,separator,limit)===3 && log==='gc'",
    );
    check(
        "let marker={};'a'.split({[Symbol.split](){return marker;}})===marker && 'a'.split({[Symbol.split](){return undefined;}})===undefined",
    );
    check(
        "let reads=0,separator={get [Symbol.split](){reads++;throw 7;}},caught=false;try{String.prototype.split.call(null,separator);}catch(e){caught=e instanceof TypeError;}caught && reads===0",
    );
    check(
        "let receiver={toString(){throw 8;}},caught=false;try{String.prototype.split.call(receiver,{get [Symbol.split](){throw 7;}});}catch(e){caught=e===7;}caught",
    );
    check(
        "let calls=0,receiver={toString(){calls++;throw 8;}},caught=false;try{String.prototype.split.call(receiver,{[Symbol.split]:7});}catch(e){caught=e instanceof TypeError;}caught && calls===0",
    );
    check("'a,b'.split({[Symbol.split]:null,toString(){return ',';}}).join('|')==='a|b'");
}

#[test]
fn edition_seventeen_ignores_primitive_prototype_split_hooks() {
    for (prototype, separator, source, expected) in [
        ("String.prototype", "','", "a,b", "a|b"),
        ("Number.prototype", "1", "a1b", "a|b"),
        ("Boolean.prototype", "true", "atrueb", "a|b"),
        ("BigInt.prototype", "1n", "a1b", "a|b"),
    ] {
        check(&format!(
            "Object.defineProperty({prototype},Symbol.split,{{get(){{throw 7;}}}});'{source}'.split({separator}).join('|')==='{expected}'"
        ));
    }
    check(
        "Object.defineProperty(Symbol.prototype,Symbol.split,{get(){throw 7;}});let caught=false;try{'abc'.split(Symbol());}catch(e){caught=e instanceof TypeError;}caught",
    );
    check(
        "Number.prototype[Symbol.split]=function(o,l){'use strict';return this.valueOf()+o.length+l;};'abc'.split(Object(1),2)===6",
    );
}

#[test]
fn fallback_conversions_run_receiver_then_limit_then_separator_even_at_zero() {
    check(
        "let log='',receiver={toString(){log+='s';return 'a,b';}},separator={get [Symbol.split](){log+='g';return undefined;},toString(){log+='r';return ',';},valueOf(){throw 7;}},limit={valueOf(){log+='l';return 0;},toString(){throw 8;}};String.prototype.split.call(receiver,separator,limit).length===0 && log==='gslr'",
    );
    check(
        "let log='',separator={toString(){log+='r';throw 7;}},limit={valueOf(){log+='l';return 0;}},caught=false;try{'a'.split(separator,limit);}catch(e){caught=e===7;}caught && log==='lr'",
    );
    check(
        "let log='',receiver={toString(){log+='s';throw 7;}},separator={toString(){log+='r';throw 8;}},limit={valueOf(){log+='l';throw 9;}},caught=false;try{String.prototype.split.call(receiver,separator,limit);}catch(e){caught=e===7;}caught && log==='s'",
    );
    check(
        "let reads=0,separator={toString(){reads++;throw 8;}},caught=false;try{'a'.split(separator,{valueOf(){throw 7;}});}catch(e){caught=e===7;}caught && reads===0",
    );
}

#[test]
fn intrinsic_results_bypass_setters_and_survive_deletion_and_collection() {
    check(
        "let P=Array.prototype;Object.defineProperty(P,'0',{set(){throw 7;}});globalThis.Array=function(){throw 8;};let a='a,b'.split(','),d=Object.getOwnPropertyDescriptor(a,'0');Object.getPrototypeOf(a)===P && d.value==='a' && d.writable && d.enumerable && d.configurable && a[1]==='b'",
    );
    check("let a='x,'.repeat(4000).split(',');a.length===4001 && a[3999]==='x' && a[4000]===''");
    check(
        "let d=Object.getOwnPropertyDescriptor(String.prototype,'split'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='split' && !n.writable && !n.enumerable && n.configurable && l.value===2 && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')",
    );
    check(
        "let caught=false;try{new String.prototype.split();}catch(e){caught=e instanceof TypeError;}caught",
    );
    let mut realm = Realm::default();
    realm.eval("let split=String.prototype.split;delete String.prototype.split;delete globalThis.String;delete globalThis.Array").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("'a'.split===undefined && split.call('a,b',',').join('|')==='a|b'"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn opted_in_host_failures_skip_pending_handlers() {
    for limits in [
        Limits {
            max_properties: Some(100),
            ..Limits::default()
        },
        Limits {
            max_steps: Some(5000),
            ..Limits::default()
        },
    ] {
        let mut realm = Realm::new(limits);
        realm.eval("let flag=0,source='x,'.repeat(200)").unwrap();
        assert!(matches!(
            realm.eval("try{source.split(',');}catch{flag=1;}finally{flag=2;}"),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{'a'.split({[Symbol.split](){Proxy;}});}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
