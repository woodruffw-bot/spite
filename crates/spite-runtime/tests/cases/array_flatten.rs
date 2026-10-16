//! Species-aware flattening with ordered sparse visits and one-level mapping.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn flat_uses_array_brand_and_compacts_holes_at_each_flattened_level() {
    check(
        "let a=[1,,[2,,[3]],undefined];let b=a.flat();b.length===4 && b[0]===1 && b[1]===2 && b[2]===a[2][2] && Object.hasOwn(b,'3') && b[3]===undefined && a.length===4",
    );
    check(
        "let a=[1,,[2,,[3]]];let b=a.flat(0);b.length===2 && b[1]===a[2] && a.flat(2).join(',')==='1,2,3' && a.flat(Infinity).join(',')==='1,2,3'",
    );
    check(
        "let fake={0:9,length:1,[Symbol.isConcatSpreadable]:true},a=[7];a[Symbol.isConcatSpreadable]=false;let b=[fake,a].flat();b.length===2 && b[0]===fake && b[1]===7",
    );
    check(
        "let a=[1];Object.setPrototypeOf(a,null);let b=Array.prototype.flat.call([a]);b.length===1 && b[0]===1",
    );
    check(
        "Array.prototype.flat.call('ab').join(',')==='a,b' && Array.prototype.flat.call({0:[1,[2]],2:3,length:3}).flat().join(',')==='1,2,3'",
    );
}

#[test]
fn depth_conversion_precedes_species_and_follows_length() {
    for depth in ["undefined", "1.9", "'1'", "true"] {
        check(&format!(
            "let a=[[1,[2]]],b=a.flat({depth});b.length===2 && b[0]===1 && b[1]===a[0][1]"
        ));
    }
    for depth in ["0", "-1", "NaN", "-Infinity", "null", "false"] {
        check(&format!(
            "let a=[[1]],b=a.flat({depth});b.length===1 && b[0]===a[0]"
        ));
    }
    check(
        "let log='',a=[[1]];Object.defineProperty(a,'constructor',{get(){log+='c';return Array;}});let d={valueOf(){log+='d';a[0]=[2];return 1;}};a.flat(d)[0]===2 && log==='dc'",
    );
    check(
        "let log='',o={get length(){log+='l';throw 7;}},d={valueOf(){log+='d';return 1;}};let caught=false;try{Array.prototype.flat.call(o,d);}catch(e){caught=e===7;}caught && log==='l'",
    );
    for source in [
        "[].flat(Symbol())",
        "[].flat(1n)",
        "[].flat(Object.create(null))",
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
fn visits_observe_live_inherited_properties_and_nested_length_snapshots() {
    check(
        "let child=[1,,3];Object.defineProperty(child,'0',{get(){child[1]=2;child[3]=4;return 1;}});let a=[child];let b=a.flat();b.join(',')==='1,2,3' && child.length===4",
    );
    check(
        "let proto={1:[2]},a=[1,,3];Object.setPrototypeOf(a,proto);Object.defineProperty(a,'0',{get(){a[2]=[4];a[3]=[5];return 1;}});let b=Array.prototype.flat.call(a);b.join(',')==='1,2,4' && a.length===4",
    );
}

#[test]
fn custom_species_gets_zero_and_has_no_final_length_assignment() {
    check(
        "let a=[[1],2],n,argc,target={length:99};function C(x){n=x;argc=arguments.length;return target;}a.constructor={[Symbol.species]:C};let b=a.flat();b===target && b[0]===1 && b[1]===2 && b.length===99 && n===0 && argc===1",
    );
    check(
        "let a=[[1]],writes=0,target={set length(x){writes++;throw 7;}};function C(){return target;}a.constructor={[Symbol.species]:C};a.flat()===target && target[0]===1 && writes===0",
    );
    check(
        "let a=[1,,[2]],target=a;function C(){return target;}a.constructor={[Symbol.species]:C};a.flat()===a && a.length===3 && a[0]===1 && a[1]===2 && Array.isArray(a[2])",
    );
    check("let o={length:1,0:[1],get constructor(){throw 7;}};Array.prototype.flat.call(o)[0]===1");
}

#[test]
fn failed_definitions_keep_prior_effects_and_bypass_inherited_setters() {
    check(
        "let a=[[1],2],target={};Object.defineProperty(target,'1',{value:9,writable:false});function C(){return target;}a.constructor={[Symbol.species]:C};let caught=false;try{a.flat();}catch(e){caught=e instanceof TypeError;}caught && target[0]===1 && target[1]===9",
    );
    check(
        "let writes=0,target=Object.create({set 0(x){writes++;}}),a=[[1]];function C(){return target;}a.constructor={[Symbol.species]:C};a.flat()===target && target[0]===1 && Object.hasOwn(target,'0') && writes===0",
    );
}

#[test]
fn flat_map_calls_only_for_present_original_elements_and_flattens_once() {
    check(
        "let a=[1,,3],calls=0,context={};let b=a.flatMap(function(v,i,o){'use strict';calls++;if(this!==context || o!==a || arguments.length!==3)throw 7;return [v,,[i]];},context);calls===2 && b.length===4 && b[0]===1 && b[1][0]===0 && b[2]===3 && b[3][0]===2",
    );
    check(
        "let a=[1,2,3],calls='';let b=a.flatMap((v,i,o)=>{calls+=i;delete o[1];o[2]=4;o[3]=5;return [v];});b.join(',')==='1,4' && calls==='02'",
    );
    check(
        "let fake={0:1,length:1,[Symbol.isConcatSpreadable]:true};[0].flatMap(()=>fake)[0]===fake",
    );
    check(
        "let a=[1],target={length:7};function C(){return target;}a.constructor={[Symbol.species]:C};a.flatMap(()=>[2,3])===target && target.length===7 && target[0]===2 && target[1]===3",
    );
    check("[1].flatMap(function(){'use strict';return [this];},false)[0]===false");
}

#[test]
fn flat_map_validates_callback_before_species_and_preserves_abrupt_results() {
    check(
        "let read=false,a=[];Object.defineProperty(a,'constructor',{get(){read=true;throw 7;}});let caught=false;try{a.flatMap(null);}catch(e){caught=e instanceof TypeError;}caught && !read",
    );
    check(
        "let log='',o={get length(){log+='l';throw 7;}};let caught=false;try{Array.prototype.flatMap.call(o,null);}catch(e){caught=e===7;}caught && log==='l'",
    );
    check(
        "let a=[1,2],target={};function C(){return target;}a.constructor={[Symbol.species]:C};let caught=false;try{a.flatMap((v)=>{if(v===2)throw 7;return [v];});}catch(e){caught=e===7;}caught && target[0]===1 && !Object.hasOwn(target,'1')",
    );
}

#[test]
fn intrinsics_have_standard_metadata_and_are_not_constructors() {
    check(
        "Array.prototype.flat.length===0 && Array.prototype.flatMap.length===1 && Array.prototype.flat.name==='flat' && Array.prototype.flatMap.name==='flatMap'",
    );
    check(
        "let a=Object.getOwnPropertyDescriptor(Array.prototype,'flat'),b=Object.getOwnPropertyDescriptor(Array.prototype,'flatMap');a.writable && a.configurable && !a.enumerable && b.writable && b.configurable && !b.enumerable",
    );
    for source in ["new Array.prototype.flat", "new Array.prototype.flatMap"] {
        assert!(matches!(
            Realm::default().eval(source),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn deep_flattening_uses_an_explicit_stack_and_cycles_obey_opted_in_work_limits() {
    std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(|| {
        check("let a=1;for(var i=0;i<4000;i++)a=[a];let b=a.flat(Infinity);b.length===1 && b[0]===1 && a.flat(1e100)[0]===1");
    }).unwrap().join().unwrap();
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    assert!(matches!(
        realm.eval("let a=[];a[0]=a;try{a.flat(Infinity);}catch{throw 7;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("1"), Ok(Value::Number(1.0)));
}
