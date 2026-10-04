//! Array slice: ordered ranges, sparse copies, species, and strict length writes.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn copies_are_shallow_preserve_holes_and_read_inherited_elements() {
    check(
        "let o={},a=[o,,3,,],b=a.slice();b!==a && b.length===4 && b[0]===o && !(1 in b) && b[2]===3 && !(3 in b)",
    );
    check(
        "let a=[1,,3];Array.prototype[1]=9;let b=a.slice(1);Object.hasOwn(b,'0') && b[0]===9 && b[1]===3 && b.length===2",
    );
    check("let a=[undefined],b=a.slice();Object.hasOwn(b,'0') && b[0]===undefined");
    check(
        "let a=[7,8],b=a.slice();let d=Object.getOwnPropertyDescriptor(b,'0');d.value===7 && d.writable && d.enumerable && d.configurable",
    );
}

#[test]
fn relative_ranges_handle_omission_fractions_nan_and_infinities() {
    for (args, expected) in [
        ("", "0,1,2,3"),
        ("undefined,undefined", "0,1,2,3"),
        ("1", "1,2,3"),
        ("1,null", ""),
        ("-3,-1", "1,2"),
        ("-99,99", "0,1,2,3"),
        ("NaN,2.9", "0,1"),
        ("1.9,-1.9", "1,2"),
        ("-0,2", "0,1"),
        ("-Infinity,Infinity", "0,1,2,3"),
        ("Infinity", ""),
        ("0,-Infinity", ""),
        ("3,1", ""),
        ("2,NaN", ""),
    ] {
        check(&format!("[0,1,2,3].slice({args}).join()==='{expected}'"));
    }
}

#[test]
fn conversions_follow_length_and_precede_species_or_element_reads() {
    check(
        "let log='',o={get length(){log+='l';return {valueOf(){log+='n';return 3;}};},get constructor(){throw 7;},get 1(){log+='g';return 9;}};let s={valueOf(){log+='s';return 1;}},e={valueOf(){log+='e';return 2;}};let b=Array.prototype.slice.call(o,s,e);log==='lnseg' && b[0]===9 && b.length===1",
    );
    check(
        "let log='',a=[1,2,3],receiver;function C(n){log+='C'+n;Object.defineProperty(this,'length',{set:function(v){log+='L'+v;receiver=this;}});}let holder={get [Symbol.species](){log+='p';return C;}};Object.defineProperty(a,'constructor',{get(){log+='c';return holder;}});Object.defineProperty(a,'1',{get(){log+='g';return 9;}});let b=a.slice({valueOf(){log+='s';return 1;}},{valueOf(){log+='e';return 2;}});log==='secpC1gL1' && b[0]===9 && receiver===b",
    );
    for args in ["1n", "0,1n", "Symbol()", "0,Symbol()"] {
        check(&format!(
            "let a=[],hits=0;Object.defineProperty(a,'constructor',{{get(){{hits++;throw 7;}}}});let caught=false;try{{a.slice({args});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
        ));
    }
    assert_eq!(
        Realm::default()
            .eval("Array.prototype.slice.call({get length(){throw 7;}},{valueOf(){throw 8;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
}

#[test]
fn mutations_use_the_original_length_and_live_source_presence() {
    check(
        "let a=[1,2,3];let b=a.slice({valueOf(){a.length=1;return 0;}},{valueOf(){a[3]=4;return 99;}});b.length===3 && b[0]===1 && !(1 in b) && !(2 in b)",
    );
    check(
        "let a=[1,2,3],log='';Object.defineProperty(a,'0',{get(){log+='a';delete a[1];a[2]=9;return 7;}});let b=a.slice();b.length===3 && b[0]===7 && !(1 in b) && b[2]===9 && log==='a'",
    );
    check(
        "let a=[1,,3];function C(){a[1]=9;delete a[2];}a.constructor={[Symbol.species]:C};let b=a.slice();b.length===3 && b[0]===1 && b[1]===9 && !(2 in b)",
    );
}

#[test]
fn species_receives_count_and_can_return_arbitrary_or_original_objects() {
    check(
        "let a=[1,2,3,4],n,argc,target;function C(x){n=x;argc=arguments.length;target=new.target;}a.constructor={[Symbol.species]:C};let b=a.slice(1,-1);b instanceof C && b[0]===2 && b[1]===3 && b.length===2 && n===2 && argc===1 && target===C",
    );
    check(
        "let a=[1,2,3],o={0:9,1:8};function C(){return o;}a.constructor={[Symbol.species]:C};delete a[0];delete a[1];a.slice()===o && o[0]===9 && o[1]===8 && o[2]===3 && o.length===3",
    );
    check(
        "let a=[1,2,3];function C(){return a;}a.constructor={[Symbol.species]:C};a.slice(1)===a && a.length===2 && a[0]===2 && a[1]===3 && !(2 in a)",
    );
    check(
        "let a=[1],positive=false;function C(n){positive=Object.is(n,0);return {};}a.constructor={[Symbol.species]:C};let b=a.slice(-0,-0);positive && b.length===0",
    );
    for value in ["undefined", "{}", "{[Symbol.species]:null}"] {
        check(&format!(
            "let a=[7],p=Array.prototype;a.constructor={value};Array=function(){{throw 7;}};let b=a.slice();b[0]===7 && Object.getPrototypeOf(b)===p"
        ));
    }
}

#[test]
fn empty_results_still_construct_and_strictly_assign_length() {
    check(
        "let calls=0,n;function C(x){calls++;n=x;}let a=[1];a.constructor={[Symbol.species]:C};let b=a.slice(1,0);calls===1 && n===0 && b instanceof C && b.length===0",
    );
    for args in ["", "1,0"] {
        check(&format!(
            "let a=[],o={{}},caught=false;Object.defineProperty(o,'length',{{value:0,writable:false}});function C(){{return o;}}a.constructor={{[Symbol.species]:C}};try{{a.slice({args});}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    check(
        "let a=[7,8],receiver,n,seen;function C(){}Object.defineProperty(C.prototype,'length',{set:function(v){receiver=this;n=v;seen=this[1];}});a.constructor={[Symbol.species]:C};let b=a.slice();receiver===b && n===2 && seen===8 && !Object.hasOwn(b,'length')",
    );
}

#[test]
fn abrupt_definitions_getters_and_length_writes_keep_prior_effects() {
    check(
        "let a=[7,8,9],o={},reads=0,caught=false;Object.defineProperty(o,'1',{value:0,configurable:false});Object.defineProperty(a,'2',{get(){reads++;return 9;}});function C(){return o;}a.constructor={[Symbol.species]:C};try{a.slice();}catch(e){caught=e instanceof TypeError;}caught && o[0]===7 && o[1]===0 && reads===0 && !Object.hasOwn(o,'length')",
    );
    check(
        "let a=[7,8,9],o,caught=false;function C(){o=this;}a.constructor={[Symbol.species]:C};Object.defineProperty(a,'1',{get(){throw 7;}});try{a.slice();}catch(e){caught=e===7;}caught && o[0]===7 && !Object.hasOwn(o,'1') && !Object.hasOwn(o,'length')",
    );
    check(
        "let a=[7,8],o={},caught=false;Object.defineProperty(o,'length',{set(){throw 9;}});function C(){return o;}a.constructor={[Symbol.species]:C};try{a.slice();}catch(e){caught=e===9;}caught && o[0]===7 && o[1]===8",
    );
    check(
        "let a=[7],o=Object.freeze([]),caught=false;function C(){return o;}a.constructor={[Symbol.species]:C};try{a.slice();}catch(e){caught=e instanceof TypeError;}caught && o.length===0",
    );
}

#[test]
fn own_data_definitions_bypass_setters_and_replace_configurable_properties() {
    check(
        "let a=[7],hits=0;Object.defineProperty(Array.prototype,'0',{set(){hits++;throw 7;},configurable:true});a.slice()[0]===7 && hits===0",
    );
    check(
        "let a=[7],o={};Object.defineProperty(o,'0',{get(){throw 7;},configurable:true});function C(){return o;}a.constructor={[Symbol.species]:C};let b=a.slice(),d=Object.getOwnPropertyDescriptor(b,'0');b===o && d.value===7 && d.writable && d.enumerable && d.configurable",
    );
}

#[test]
fn generic_receivers_preserve_utf16_and_full_width_source_indices() {
    check(
        "let b=Array.prototype.slice.call('💩x',1);b.length===2 && b[0]==='\\uDCA9' && b[1]==='x'",
    );
    check(
        "Array.prototype.slice.call(true).length===0 && Array.prototype.slice.call(7).length===0",
    );
    check(
        "let o={length:4294967297,4294967296:7,get constructor(){throw 7;}};let b=Array.prototype.slice.call(o,-1);b.length===1 && b[0]===7",
    );
    check(
        "let o={length:Infinity,9007199254740990:7};let b=Array.prototype.slice.call(o,-1);b.length===1 && b[0]===7",
    );
    for receiver in ["null", "undefined", "{length:1n}"] {
        assert!(matches!(
            Realm::default().eval(&format!("Array.prototype.slice.call({receiver})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn invalid_species_and_huge_ranges_fail_before_copying() {
    for constructor in [
        "null",
        "7",
        "{[Symbol.species]:()=>1}",
        "{[Symbol.species]:7}",
    ] {
        check(&format!(
            "let a=[7],reads=0;a.constructor={constructor};Object.defineProperty(a,'0',{{get(){{reads++;return 7;}}}});let caught=false;try{{a.slice();}}catch(e){{caught=e instanceof TypeError;}}caught && reads===0"
        ));
    }
    check(
        "let o={length:4294967296,get 0(){throw 7;}},caught=false;try{Array.prototype.slice.call(o);}catch(e){caught=e instanceof RangeError;}caught",
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(5_000),
        ..Limits::default()
    });
    assert!(matches!(
        realm.eval("try{Array(4294967295).slice();}catch{throw 7;}"),
        Err(Error::Limit { .. })
    ));
}

#[test]
fn function_metadata_and_object_edges_survive_collection() {
    check(
        "let f=Array.prototype.slice,d=Object.getOwnPropertyDescriptor(Array.prototype,'slice');f.name==='slice' && f.length===2 && f.prototype===undefined && d.value===f && d.writable && !d.enumerable && d.configurable",
    );
    assert!(matches!(
        Realm::default().eval("new Array.prototype.slice"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm
        .eval("let f=Array.prototype.slice,o={},a=[o].slice();delete Array.prototype.slice")
        .unwrap();
    realm.collect(10_000).unwrap();
    assert_eq!(
        realm.eval("a[0]===o && f.call([o])[0]===o"),
        Ok(Value::Boolean(true))
    );
}
