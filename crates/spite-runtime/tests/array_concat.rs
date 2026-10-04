//! Concatenation preserves ordered hooks, sparse visits, and partial effects.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn arrays_spread_one_level_and_preserve_holes_and_identity() {
    check(
        "let o={},a=[1,,o],nested=[2],b=a.concat([nested,,3],undefined);b!==a && b.length===7 && b[0]===1 && !(1 in b) && b[2]===o && b[3]===nested && !(4 in b) && b[5]===3 && Object.hasOwn(b,'6') && b[6]===undefined && a.length===3",
    );
    check("let a=[1,,3],b=a.concat();b!==a && b.length===3 && !(1 in b) && b[2]===3");
    check("let a=[,,].concat([,,]);a.length===4 && !(0 in a) && !(3 in a)");
    check(
        "let b=[].concat(null,true,7,'x',1n,Symbol());b.length===6 && b[0]===null && b[3]==='x' && b[4]===1n && typeof b[5]==='symbol'",
    );
}

#[test]
fn spreadability_uses_the_symbol_hook_then_internal_array_brand() {
    check(
        "let a=[1],o={0:7,length:1,[Symbol.isConcatSpreadable]:true};a[Symbol.isConcatSpreadable]=false;let b=a.concat(o);b.length===2 && b[0]===a && b[1]===7",
    );
    for marker in ["false", "null", "0", "''", "NaN"] {
        check(&format!(
            "let a=[7];a[Symbol.isConcatSpreadable]={marker};let b=[].concat(a);b.length===1 && b[0]===a"
        ));
    }
    for marker in ["true", "1", "'x'", "{}", "1n", "Symbol()"] {
        check(&format!(
            "let o={{0:7,length:1,[Symbol.isConcatSpreadable]:{marker}}};[].concat(o)[0]===7"
        ));
    }
    check("let a=[7];a[Symbol.isConcatSpreadable]=undefined;[].concat(a)[0]===7");
    check(
        "let o=Object.create(Array.prototype);o[0]=7;let b=[].concat(o);b.length===1 && b[0]===o",
    );
    check("let a=[7];Object.setPrototypeOf(a,null);Array.prototype.concat.call([],a)[0]===7");
    check(
        "let o={valueOf(){throw 7;}};[].concat({0:8,length:1,[Symbol.isConcatSpreadable]:o})[0]===8",
    );
}

#[test]
fn primitive_arguments_are_unboxed_and_receivers_are_boxed_once() {
    check(
        "let b=Array.prototype.concat.call('💩',7);b.length===2 && b[0] instanceof String && b[0].valueOf()==='💩' && b[1]===7",
    );
    check(
        "let b=Array.prototype.concat.call(false);b.length===1 && b[0] instanceof Boolean && b[0].valueOf()===false",
    );
    check(
        "let hits=0;Object.defineProperty(Number.prototype,Symbol.isConcatSpreadable,{get(){hits++;return true;}});Number.prototype.length=1;Number.prototype[0]=8;let b=[].concat(7),c=Array.prototype.concat.call(7);hits===1 && b[0]===7 && c[0]===8",
    );
    for receiver in ["null", "undefined"] {
        assert!(matches!(
            Realm::default().eval(&format!("Array.prototype.concat.call({receiver})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn species_construction_precedes_spreadability_and_length() {
    check(
        "let log='',a=[7],n,argc,target;function C(x){log+='C';n=x;argc=arguments.length;target=new.target;}Object.defineProperty(a,'constructor',{get(){log+='c';return {get [Symbol.species](){log+='p';return C;}};}});Object.defineProperty(a,Symbol.isConcatSpreadable,{get(){log+='s';return true;}});Object.defineProperty(a,'0',{get(){log+='g';return 7;}});let o={get [Symbol.isConcatSpreadable](){log+='t';return true;},get length(){log+='l';return 1;},get 0(){log+='h';return 8;}};let b=a.concat(o);log==='cpCsgtlh' && n===0 && argc===1 && target===C && b instanceof C && b[0]===7 && b[1]===8 && b.length===2",
    );
    check(
        "let a=[1],n;function C(x){n=x;a.length=0;}a.constructor={[Symbol.species]:C};let b=a.concat();n===0 && b.length===0 && !(0 in b)",
    );
    check(
        "let o={0:7,length:1,[Symbol.isConcatSpreadable]:true,get constructor(){throw 7;}};Array.prototype.concat.call(o)[0]===7",
    );
    check(
        "let a=[7],p=Array.prototype;a.constructor={[Symbol.species]:null};Array=function(){throw 7;};let b=a.concat();b[0]===7 && Object.getPrototypeOf(b)===p",
    );
}

#[test]
fn constructors_and_spreadability_fail_before_later_reads() {
    for setup in [
        "Object.defineProperty(a,'constructor',{get(){throw 7;}})",
        "a.constructor={get [Symbol.species](){throw 7;}}",
        "a.constructor={[Symbol.species]:function(){throw 7;}}",
    ] {
        check(&format!(
            "let a=[],hits=0;{setup};Object.defineProperty(a,Symbol.isConcatSpreadable,{{get(){{hits++;throw 8;}}}});let caught=false;try{{a.concat();}}catch(e){{caught=e===7;}}caught && hits===0"
        ));
    }
    for constructor in ["null", "7", "{[Symbol.species]:()=>1}"] {
        check(&format!(
            "let a=[],hits=0;a.constructor={constructor};Object.defineProperty(a,Symbol.isConcatSpreadable,{{get(){{hits++;}}}});let caught=false;try{{a.concat();}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
        ));
    }
    check(
        "let o={get [Symbol.isConcatSpreadable](){throw 7;},get length(){throw 8;}},caught=false;try{[].concat(o);}catch(e){caught=e===7;}caught",
    );
    check(
        "let o={get length(){throw 7;},get 0(){throw 8;},[Symbol.isConcatSpreadable]:true},caught=false;try{[].concat(o);}catch(e){caught=e===7;}caught",
    );
    check("let o={get length(){throw 7;},[Symbol.isConcatSpreadable]:false};[].concat(o)[0]===o");
}

#[test]
fn sparse_visits_are_live_and_inherited_values_are_copied() {
    check(
        "let a=[1,,3],log='';Array.prototype[1]=2;Object.defineProperty(a,'0',{get(){log+='a';a[2]=9;a[3]=4;return 7;}});let b=a.concat();b.length===3 && b.join()==='7,2,9' && Object.hasOwn(b,'1') && log==='a'",
    );
    check(
        "let a=[1,2,3];Object.defineProperty(a,'0',{get(){delete a[1];a.length=1;return 7;}});let b=a.concat();b.length===3 && b[0]===7 && !(1 in b) && !(2 in b)",
    );
    check(
        "let a=[1,,3];Object.defineProperty(a,'0',{get(){a[1]=2;return 1;}});a.concat().join()==='1,2,3'",
    );
    check(
        "let b=[7];let a={0:1,length:1,[Symbol.isConcatSpreadable]:true,get 0(){b[0]=8;return 1;}};[].concat(a,b).join()==='1,8'",
    );
}

#[test]
fn lengths_convert_in_order_and_only_spread_objects_are_array_like() {
    check(
        "let log='',o={0:7,[Symbol.isConcatSpreadable]:true,get length(){log+='l';return {valueOf(){log+='v';return '1.9';}};}};[].concat(o)[0]===7 && log==='lv'",
    );
    for length in ["-1", "NaN", "undefined", "null"] {
        check(&format!(
            "[].concat({{length:{length},get 0(){{throw 7;}},[Symbol.isConcatSpreadable]:true}}).length===0"
        ));
    }
    for length in ["1n", "Symbol()"] {
        check(&format!(
            "let caught=false;try{{[].concat({{length:{length},get 0(){{throw 7;}},[Symbol.isConcatSpreadable]:true}});}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    check("let o={length:1,0:7};let b=[].concat(o);b.length===1 && b[0]===o");
}

#[test]
fn result_definitions_bypass_setters_and_preserve_custom_holes() {
    check(
        "let hits=0,a=[7];Object.defineProperty(Array.prototype,'0',{set(){hits++;throw 7;},configurable:true});a.concat()[0]===7 && hits===0",
    );
    check(
        "let a=[7,,9],o={1:8};Object.defineProperty(o,'0',{get(){throw 7;},configurable:true});function C(){return o;}a.constructor={[Symbol.species]:C};let b=a.concat(),d=Object.getOwnPropertyDescriptor(b,'0');b===o && b[0]===7 && b[1]===8 && b[2]===9 && b.length===3 && d.writable && d.enumerable && d.configurable",
    );
    check(
        "let a=[7],o={};Object.defineProperty(o,'0',{value:0,writable:false,configurable:true});function C(){return o;}a.constructor={[Symbol.species]:C};let b=a.concat(),d=Object.getOwnPropertyDescriptor(b,'0');d.value===7 && d.writable && d.enumerable && d.configurable",
    );
}

#[test]
fn abrupt_definitions_and_reads_preserve_earlier_results() {
    check(
        "let a=[7,8],o={},later=0;Object.defineProperty(o,'1',{value:0,configurable:false});function C(){return o;}a.constructor={[Symbol.species]:C};let z={get [Symbol.isConcatSpreadable](){later++;return true;}};let caught=false;try{a.concat(z);}catch(e){caught=e instanceof TypeError;}caught && o[0]===7 && o[1]===0 && !Object.hasOwn(o,'length') && later===0",
    );
    check(
        "let a=[7,8,9],o,later=0;function C(){o=this;}a.constructor={[Symbol.species]:C};Object.defineProperty(a,'1',{get(){throw 7;}});Object.defineProperty(a,'2',{get(){later++;return 9;}});let caught=false;try{a.concat();}catch(e){caught=e===7;}caught && o[0]===7 && !Object.hasOwn(o,'1') && !Object.hasOwn(o,'length') && later===0",
    );
    check(
        "let a=[7],o=Object.freeze([]);function C(){return o;}a.constructor={[Symbol.species]:C};let caught=false;try{a.concat();}catch(e){caught=e instanceof TypeError;}caught && o.length===0",
    );
}

#[test]
fn final_length_is_strict_and_observes_setters_even_for_empty_results() {
    check(
        "let a=[7,,9],o={},n,receiver,seen;Object.defineProperty(o,'length',{set(v){n=v;receiver=this;seen=this[2];}});function C(){return o;}a.constructor={[Symbol.species]:C};a.concat()===o && n===3 && receiver===o && seen===9",
    );
    check(
        "let a=[],o={},n;Object.defineProperty(o,'length',{set(v){n=v;}});function C(){return o;}a.constructor={[Symbol.species]:C};a.concat()===o && Object.is(n,0)",
    );
    check(
        "let a=[7],o={};Object.defineProperty(o,'length',{set(){throw 8;}});function C(){return o;}a.constructor={[Symbol.species]:C};let caught=false;try{a.concat();}catch(e){caught=e===8;}caught && o[0]===7",
    );
    check(
        "let a=[],o={};Object.defineProperty(o,'length',{value:0,writable:false});function C(){return o;}a.constructor={[Symbol.species]:C};let caught=false;try{a.concat();}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn species_can_return_the_source_and_later_items_observe_its_mutations() {
    check(
        "let a=[1,2];function C(){return a;}a.constructor={[Symbol.species]:C};a.concat(a)===a && a.join()==='1,2,1,2'",
    );
    check(
        "let a=[7];function C(){return a;}a.constructor={[Symbol.species]:C};a[Symbol.isConcatSpreadable]=false;a.concat()===a && a[0]===a && a.length===1",
    );
    check(
        "let a=[1,2];function C(){return a;}a.constructor={[Symbol.species]:C};let o={get [Symbol.isConcatSpreadable](){a.length=1;return false;}};a.concat(o,a)===a && a.length===6 && a[0]===1 && !(1 in a) && a[2]===o && a[3]===1 && !(4 in a) && a[5]===o",
    );
}

#[test]
fn safe_integer_overflow_precedes_indexed_reads_and_keeps_partial_effects() {
    check(
        "let a=[7],o={},reads=0;function C(){return o;}a.constructor={[Symbol.species]:C};let huge={length:Infinity,[Symbol.isConcatSpreadable]:true,get 0(){reads++;throw 7;}};let caught=false;try{a.concat(huge);}catch(e){caught=e instanceof TypeError;}caught && reads===0 && o[0]===7 && !Object.hasOwn(o,'length')",
    );
    for length in ["4294967296", "Infinity"] {
        let mut realm = Realm::new(Limits {
            max_steps: 5_000,
            ..Limits::default()
        });
        assert!(matches!(realm.eval(&format!("try{{[].concat({{length:{length},[Symbol.isConcatSpreadable]:true}});}}catch{{throw 7;}}")), Err(Error::Limit { .. })));
    }
}

#[test]
fn function_metadata_and_result_edges_survive_collection() {
    check(
        "let f=Array.prototype.concat,d=Object.getOwnPropertyDescriptor(Array.prototype,'concat');f.name==='concat' && f.length===1 && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable",
    );
    assert!(matches!(
        Realm::default().eval("new Array.prototype.concat"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm
        .eval("let f=Array.prototype.concat,o={},a=[o].concat(o);delete Array.prototype.concat")
        .unwrap();
    realm.collect(10_000).unwrap();
    assert_eq!(
        realm.eval("a[0]===o && a[1]===o && f.call([o],o)[1]===o"),
        Ok(Value::Boolean(true))
    );
}
