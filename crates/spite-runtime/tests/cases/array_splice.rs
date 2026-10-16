//! Sparse splice ranges, species results, strict moves, and partial effects.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn omitted_arguments_and_explicit_undefined_have_distinct_ranges() {
    for (call, remaining, removed) in [
        ("", "0,1,2,3", ""),
        ("undefined", "", "0,1,2,3"),
        ("1", "0", "1,2,3"),
        ("1,undefined", "0,1,2,3", ""),
        ("-2,1", "0,1,3", "2"),
        ("-Infinity,1", "1,2,3", "0"),
        ("Infinity,Infinity,7", "0,1,2,3,7", ""),
        ("1.9,1.9,7", "0,7,2,3", "1"),
        ("NaN,NaN,7", "7,0,1,2,3", ""),
        ("1,-1,7", "0,7,1,2,3", ""),
        ("1,Infinity,7", "0,7", "1,2,3"),
    ] {
        check(&format!(
            "let a=[0,1,2,3],b=a.splice({call});a.join(',')==='{remaining}' && b.join(',')==='{removed}'"
        ));
    }
}

#[test]
fn shrink_grow_and_equal_replacements_preserve_sparse_presence() {
    check(
        "let a=[0,,2,,4],b=a.splice(1,2);b.length===2 && !(0 in b) && b[1]===2 && a.length===3 && a[0]===0 && !(1 in a) && a[2]===4",
    );
    check(
        "let a=[0,,2],b=a.splice(1,0,7,8);b.length===0 && a.length===5 && a[0]===0 && a[1]===7 && a[2]===8 && !(3 in a) && a[4]===2",
    );
    check(
        "let a=[0,,2];Object.defineProperty(a,'2',{get(){throw 7;}});let b=a.splice(0,2,7,8);b.length===2 && b[0]===0 && !(1 in b) && a[0]===7 && a[1]===8 && a.length===3",
    );
    check(
        "let a=[undefined,,2],b=a.splice(0,2);Object.hasOwn(b,'0') && b[0]===undefined && !Object.hasOwn(b,'1') && a.length===1 && a[0]===2",
    );
    check(
        "let a=[0,,2,,4],proto={1:1,3:3};Object.setPrototypeOf(a,proto);let b=Array.prototype.splice.call(a,1,2);b.join(',')==='1,2' && a.length===3 && Object.hasOwn(a,'1') && a[1]===3 && a[2]===4",
    );
}

#[test]
fn length_and_argument_conversions_precede_species_and_indexed_reads() {
    check(
        "let log='',a=[0,1,2];Object.defineProperty(a,'constructor',{get(){log+='c';return Array;}});let start={valueOf(){log+='s';a[1]=7;return 1;}},count={valueOf(){log+='d';a[2]=8;return 1;}};let b=a.splice(start,count);log==='sdc' && b[0]===7 && a.join(',')==='0,8'",
    );
    check(
        "let log='',o={get length(){log+='l';return {valueOf(){log+='n';return 0;}}},set length(x){log+='L';}},s={valueOf(){log+='s';return 0;}},d={valueOf(){log+='d';return 0;}};Array.prototype.splice.call(o,s,d).length===0 && log==='lnsdL'",
    );
    check(
        "let log='',o={get length(){log+='l';throw 7;}},s={valueOf(){log+='s';return 0;}};let caught=false;try{Array.prototype.splice.call(o,s);}catch(e){caught=e===7;}caught && log==='l'",
    );
    for source in [
        "[].splice(Symbol())",
        "[].splice(0,1n)",
        "Array.prototype.splice.call(null)",
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
fn generic_receivers_use_live_gets_and_ordered_setters() {
    check(
        "let log='',o={get length(){log+='l';return 3;},set length(x){log+='L'+x;},get 1(){log+='g1';return 2;},set 1(x){log+='s1'+x;},get 2(){log+='g2';return 3;},set 2(x){log+='s2'+x;}};let b=Array.prototype.splice.call(o,1,1,7,8);b[0]===2 && o[3]===3 && log==='lg1g2s17s28L4'",
    );
    check(
        "let o={0:0,1:1,2:2,length:3,get constructor(){throw 7;}};let b=Array.prototype.splice.call(o,1,1,8);b[0]===1 && o[1]===8 && o[2]===2 && o.length===3",
    );
    check(
        "let a=[0,1,2,3];Object.defineProperty(a,'1',{get(){a[3]=7;delete a[2];return 1;},configurable:true});let b=a.splice(0,3);b.length===3 && b[0]===0 && b[1]===1 && !(2 in b) && a[0]===7 && a.length===1",
    );
}

#[test]
fn species_gets_delete_count_and_finishes_its_length_before_source_moves() {
    check(
        "let a=[0,1,2],target={length:99},n,argc;function C(x){n=x;argc=arguments.length;return target;}a.constructor={[Symbol.species]:C};let b=a.splice(1,1,7);b===target && b[0]===1 && b.length===1 && n===1 && argc===1 && a.join(',')==='0,7,2'",
    );
    check(
        "let a=[0,1,2],target={set length(x){a[2]=8;}};function C(){return target;}a.constructor={[Symbol.species]:C};let b=a.splice(1,1);b===target && b[0]===1 && a.join(',')==='0,8'",
    );
    check(
        "let a=[0,1,2,3];function C(){return a;}a.constructor={[Symbol.species]:C};let b=a.splice(1,2);b===a && a.length===2 && a[0]===1 && !(1 in a)",
    );
    check(
        "let a=[0,,2],target={0:9,length:99};function C(){return target;}a.constructor={[Symbol.species]:C};a.splice(1,1)===target && target[0]===9 && target.length===1 && a.join(',')==='0,2'",
    );
}

#[test]
fn result_definitions_bypass_setters_and_fail_before_source_mutation() {
    check(
        "let writes=0,target=Object.create({set 0(x){writes++;}}),a=[0,1,2];function C(){return target;}a.constructor={[Symbol.species]:C};let b=a.splice(1,1);b[0]===1 && Object.hasOwn(b,'0') && writes===0 && a.join(',')==='0,2'",
    );
    check(
        "let a=[0,1,2],target={};Object.defineProperty(target,'1',{value:9});function C(){return target;}a.constructor={[Symbol.species]:C};let caught=false;try{a.splice(0,2);}catch(e){caught=e instanceof TypeError;}caught && target[0]===0 && target[1]===9 && a.join(',')==='0,1,2'",
    );
    check(
        "let a=[0,1,2],target={set length(x){throw 7;}};function C(){return target;}a.constructor={[Symbol.species]:C};let caught=false;try{a.splice(1,1);}catch(e){caught=e===7;}caught && target[0]===1 && a.join(',')==='0,1,2'",
    );
}

#[test]
fn failed_moves_deletions_and_final_length_writes_keep_prior_effects() {
    check(
        "let o={0:0,1:1,2:2,length:3};Object.defineProperty(o,'3',{value:9,writable:false});let caught=false;try{Array.prototype.splice.call(o,1,0,7,8);}catch(e){caught=e instanceof TypeError;}caught && o[4]===2 && o[3]===9 && o[1]===1 && o.length===3",
    );
    check(
        "let o={0:0,1:1,2:2,3:3,4:4,length:5};Object.defineProperty(o,'3',{configurable:false});let caught=false;try{Array.prototype.splice.call(o,0,3);}catch(e){caught=e instanceof TypeError;}caught && o[0]===3 && o[1]===4 && !Object.hasOwn(o,'4') && o[3]===3 && o[2]===2 && o.length===5",
    );
    check(
        "let o={0:0,1:1,2:2};Object.defineProperty(o,'length',{value:3,writable:false});let caught=false;try{Array.prototype.splice.call(o,1,1);}catch(e){caught=e instanceof TypeError;}caught && o[1]===2 && !Object.hasOwn(o,'2') && o.length===3",
    );
    check(
        "let caught=false;try{Object.freeze([]).splice();}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn safe_integer_limits_use_full_width_indices_and_precede_indexed_reads() {
    check(
        "let o={9007199254740990:7,length:Infinity};let b=Array.prototype.splice.call(o,-1,1);b.length===1 && b[0]===7 && !Object.hasOwn(o,'9007199254740990') && o.length===9007199254740990",
    );
    check(
        "let o={9007199254740989:7,length:9007199254740990};let b=Array.prototype.splice.call(o,-1,0,8);b.length===0 && o[9007199254740989]===8 && o[9007199254740990]===7 && o.length===9007199254740991",
    );
    check(
        "let reads=0,o={length:9007199254740991,get 0(){reads++;return 7;}};let caught=false;try{Array.prototype.splice.call(o,0,0,1);}catch(e){caught=e instanceof TypeError;}caught && reads===0 && o.length===9007199254740991",
    );
    check(
        "let reads=0,o={length:4294967296,get 0(){reads++;return 7;}};let caught=false;try{Array.prototype.splice.call(o,0,Infinity);}catch(e){caught=e instanceof RangeError;}caught && reads===0",
    );
}

#[test]
fn strings_reject_strict_mutation_and_empty_calls_still_set_source_length() {
    check(
        "let writes=0,o={get length(){return 0;},set length(x){writes++;if(x!==0)throw 7;}};Array.prototype.splice.call(o).length===0 && writes===1",
    );
    check(
        "let caught=false;try{Array.prototype.splice.call('abc',1,1);}catch(e){caught=e instanceof TypeError;}caught",
    );
    check("Array.prototype.splice.call(true).length===0");
}

#[test]
fn complete_array_prototype_supports_reflection_integrity_and_method_deletion() {
    check(
        "Array.prototype.splice.name==='splice' && Array.prototype.splice.length===2 && Array.prototype.splice.prototype===undefined",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(Array.prototype,'splice');d.writable && !d.enumerable && d.configurable",
    );
    check(
        "let names=Object.getOwnPropertyNames(Array.prototype);names.sort().join(',')==='at,concat,constructor,copyWithin,entries,every,fill,filter,find,findIndex,findLast,findLastIndex,flat,flatMap,forEach,includes,indexOf,join,keys,lastIndexOf,length,map,pop,push,reduce,reduceRight,reverse,shift,slice,some,sort,splice,toLocaleString,toReversed,toSorted,toSpliced,toString,unshift,values,with' && Object.getOwnPropertySymbols(Array.prototype).length===2 && Object.keys(Array.prototype).length===0",
    );
    check("Object.freeze(Array.prototype)===Array.prototype && Object.isFrozen(Array.prototype)");
    check(
        "delete Array.prototype.splice && [].splice===undefined && !Object.hasOwn(Array.prototype,'splice') && Object.getOwnPropertyDescriptor(Array.prototype,'splice')===undefined",
    );
    assert!(matches!(
        Realm::default().eval("new Array.prototype.splice()"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}
