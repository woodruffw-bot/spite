//! Concrete TypedArray constructors, factories, and intrinsic graph (23.2).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

const KINDS: [(&str, usize, bool); 12] = [
    ("Int8Array", 1, false),
    ("Uint8Array", 1, false),
    ("Uint8ClampedArray", 1, false),
    ("Int16Array", 2, false),
    ("Uint16Array", 2, false),
    ("Int32Array", 4, false),
    ("Uint32Array", 4, false),
    ("BigInt64Array", 8, true),
    ("BigUint64Array", 8, true),
    ("Float16Array", 2, false),
    ("Float32Array", 4, false),
    ("Float64Array", 8, false),
];

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn exception(source: &str, expected: ExceptionKind) {
    let result = Realm::default().eval(source);
    assert!(
        matches!(&result, Err(Error::Exception { kind, .. }) if *kind == expected),
        "{source}: {result:?}"
    );
}

#[test]
fn all_concrete_kinds_construct_fixed_zero_initialized_primitive_lengths() {
    for (name, width, bigint) in KINDS {
        let zero = if bigint { "0n" } else { "0" };
        check(&format!(
            "let C={name},a=new C,b=new C(undefined),c=new C(3.9),d=new C('2');a.length===0 && b.length===0 && c.length===3 && c.byteLength==={} && c.byteOffset===0 && c[0]==={zero} && c[2]==={zero} && d.length===2 && !c.buffer.resizable && c.buffer.byteLength===c.byteLength && a.buffer!==b.buffer && Object.getPrototypeOf(c)===C.prototype && ArrayBuffer.isView(c)",
            3 * width
        ));
    }
}

#[test]
fn all_kinds_accept_iterable_array_like_typed_array_and_buffer_overloads() {
    for (name, width, bigint) in KINDS {
        let one = if bigint { "1n" } else { "1" };
        let two = if bigint { "2n" } else { "2" };
        check(&format!(
            "let C={name},ignored={{valueOf(){{throw 7;}}}},a=new C([{one},{two}],ignored,ignored),b=new C({{length:2,0:{one},1:{two}}}),c=new C(a,ignored,ignored),buffer=new ArrayBuffer({}),d=new C(buffer,{width},2,ignored);d[0]={two};a.length===2 && a[0]==={one} && a[1]==={two} && b[0]==={one} && b[1]==={two} && c[0]==={one} && c[1]==={two} && c.buffer!==a.buffer && c.byteOffset===0 && !c.buffer.resizable && d.buffer===buffer && d.byteOffset==={width} && d.byteLength==={} && d.length===2 && d[0]==={two} && new C(2,ignored,ignored).length===2",
            5 * width,
            2 * width
        ));
    }
}

#[test]
fn primitive_strings_are_lengths_while_boxed_strings_are_iterable_sources() {
    for (name, _, bigint) in KINDS {
        let one = if bigint { "1n" } else { "1" };
        let two = if bigint { "2n" } else { "2" };
        check(&format!(
            "let C={name},a=new C('12'),b=new C(Object('12'));a.length===12 && b.length===2 && b[0]==={one} && b[1]==={two} && new C(new Number(3)).length===0 && new C(Object(3n)).length===0 && new C(null).length===0 && new C(false).length===0 && new C(true).length===1 && new C(NaN).length===0 && new C(-0.9).length===0"
        ));
    }
}

#[test]
fn constructors_require_new_and_check_index_and_byte_domains_without_wrapping() {
    for (name, width, _) in KINDS {
        check(&format!(
            "let hits=0,caught=false;try{{{name}({{get [Symbol.iterator](){{hits++;throw 7;}},valueOf(){{hits++;throw 8;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
        ));
        for value in ["-1", "Infinity", "-Infinity", "9007199254740992"] {
            exception(&format!("new {name}({value})"), ExceptionKind::RangeError);
        }
        for value in ["1n", "Symbol()"] {
            exception(&format!("new {name}({value})"), ExceptionKind::TypeError);
        }
        if width > 1 {
            exception(
                &format!("new {name}(9007199254740991)"),
                ExceptionKind::RangeError,
            );
        }
    }
    check(
        "let a=new Uint8Array(9007199254740991);a.length===9007199254740991 && a.byteLength===9007199254740991 && a[9007199254740990]===0",
    );
    check(
        "let hits=0,caught=false;try{new Float64Array({length:9007199254740991,get 0(){hits++;throw 7;}});}catch(e){caught=e instanceof RangeError;}caught && hits===0",
    );
}

#[test]
fn prototype_lookup_order_distinguishes_primitive_and_object_sources() {
    check(
        "let log='',caught=false;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';throw 7;}});try{Reflect.construct(Uint16Array,[-1],N);}catch(e){caught=e instanceof RangeError;}caught && log===''",
    );
    check(
        "let log='',p={};function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';return p;}});let source={get [Symbol.iterator](){log+='i';return undefined;},get length(){log+='l';return {valueOf(){log+='n';return 1;}};},get 0(){log+='g';return {valueOf(){log+='v';return 7;}};}};let a=Reflect.construct(Uint16Array,[source],N);log==='pilngv' && Object.getPrototypeOf(a)===p && a[0]===7",
    );
    check(
        "let log='',caught=false;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';throw 7;}});try{Reflect.construct(Float64Array,[9007199254740991],N);}catch(e){caught=e===7;}caught && log==='p'",
    );
    check(
        "let p={},hits=0;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){hits++;return p;}});let a=Reflect.construct(Uint8Array,[],N);hits===1 && Object.getPrototypeOf(a)===p && Object.getOwnPropertyDescriptor(Object.getPrototypeOf(Uint8Array.prototype),'length').get.call(a)===0",
    );
    for (name, _, _) in KINDS {
        check(&format!(
            "function C(){{}}C.prototype=7;Object.getPrototypeOf(Reflect.construct({name},[2],C))==={name}.prototype"
        ));
    }
}

#[test]
fn buffer_input_aligns_before_length_conversion_or_detachment_and_uses_current_bounds() {
    check(
        "let b=new ArrayBuffer(8),log='',caught=false;b.transfer();function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';return Uint16Array.prototype;}});try{Reflect.construct(Uint16Array,[b,{valueOf(){log+='o';return 1;}},{valueOf(){log+='l';throw 7;}}],N);}catch(e){caught=e instanceof RangeError;}caught && log==='po'",
    );
    check(
        "let b=new ArrayBuffer(8),log='',caught=false;b.transfer();try{new Uint16Array(b,{valueOf(){log+='o';return 0;}},{valueOf(){log+='l';return -1;}});}catch(e){caught=e instanceof RangeError;}caught && log==='ol'",
    );
    check(
        "let b=new ArrayBuffer(2,{maxByteLength:8}),log='';let a=new Uint16Array(b,{valueOf(){log+='o';return 0;}},{valueOf(){log+='l';b.resize(8);return 4;}});log==='ol' && a.buffer===b && a.length===4 && a.byteLength===8",
    );
    check(
        "let b=new ArrayBuffer(8,{maxByteLength:12}),caught=false;try{new Uint16Array(b,0,{valueOf(){b.resize(2);return 4;}});}catch(e){caught=e instanceof RangeError;}caught && b.byteLength===2",
    );
    check(
        "let b=new ArrayBuffer(8),log='',caught=false;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';b.transfer();return Uint16Array.prototype;}});try{Reflect.construct(Uint16Array,[b,{valueOf(){log+='o';return 0;}},{valueOf(){log+='l';return 1;}}],N);}catch(e){caught=e instanceof TypeError;}caught && log==='pol'",
    );
}

#[test]
fn buffer_overloads_distinguish_fixed_alignment_auto_lengths_and_explicit_counts() {
    exception(
        "new Uint16Array(new ArrayBuffer(3),0)",
        ExceptionKind::RangeError,
    );
    check(
        "let b=new ArrayBuffer(3),a=new Uint16Array(b,0,1),r=new ArrayBuffer(5,{maxByteLength:9}),auto=new Uint16Array(r,2),fixed=new Uint16Array(r,2,1);a.length===1 && a.byteLength===2 && auto.length===1 && auto.byteLength===2 && fixed.length===1 && (r.resize(9),auto.length===3) && fixed.length===1",
    );
    check(
        "let b=new ArrayBuffer(8);Object.defineProperty(b,'byteLength',{get(){throw 7;}});class C extends Uint16Array{}Object.defineProperty(C,'BYTES_PER_ELEMENT',{value:1});Object.defineProperty(C.prototype,'BYTES_PER_ELEMENT',{value:1});let a=new C(b,2,2);a.byteOffset===2 && a.byteLength===4 && a.length===2 && a.BYTES_PER_ELEMENT===1",
    );
    for source in [
        "new Uint16Array(new ArrayBuffer(8),9)",
        "new Uint16Array(new ArrayBuffer(8),0,5)",
        "new Uint16Array(new ArrayBuffer(8),0,9007199254740991)",
    ] {
        exception(source, ExceptionKind::RangeError);
    }
}

#[test]
fn typed_array_sources_ignore_public_hooks_and_are_validated_after_prototype_lookup() {
    check(
        "let b=new ArrayBuffer(5,{maxByteLength:8}),source=new Uint8Array(b,1);source[0]=7;source[1]=9;for(let key of ['length','byteLength','byteOffset','buffer','constructor'])Object.defineProperty(source,key,{get(){throw 7;}});Object.defineProperty(source,Symbol.iterator,{get(){throw 8;}});let copy=new Uint8Array(source);copy.length===4 && copy[0]===7 && copy[1]===9 && copy.byteOffset===0 && copy.buffer!==b && !copy.buffer.resizable",
    );
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),source=new Uint8Array(b);source[0]=7;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){b.resize(1);return Uint8Array.prototype;}});let copy=Reflect.construct(Uint8Array,[source],N);copy.length===1 && copy[0]===7",
    );
    check(
        "let b=new ArrayBuffer(4),source=new Uint8Array(b),caught=false;function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){b.transfer();return Uint8Array.prototype;}});try{Reflect.construct(Uint8Array,[source],N);}catch(e){caught=e instanceof TypeError;}caught && b.detached",
    );
    check(
        "let b=new ArrayBuffer(4,{maxByteLength:8}),source=new Uint8Array(b,1,3),caught=false;b.resize(2);try{new Uint8Array(source);}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn same_kind_copies_preserve_raw_nan_sign_and_payload_from_nonzero_offsets() {
    for (name, width, set, get, bits) in [
        ("Float16Array", 2, "setUint16", "getUint16", "65535"),
        ("Float32Array", 4, "setUint32", "getUint32", "4294967295"),
        (
            "Float64Array",
            8,
            "setBigUint64",
            "getBigUint64",
            "18446744073709551615n",
        ),
    ] {
        check(&format!(
            "let b=new ArrayBuffer({}),bytes=new DataView(b);bytes.{set}({width},{bits});let source=new {name}(b,{width},1),copy=new {name}(source);Number.isNaN(source[0]) && Number.isNaN(copy[0]) && copy.byteOffset===0 && copy.buffer!==b && copy.buffer.byteLength==={width} && new DataView(copy.buffer).{get}(0)==={bits}",
            3 * width
        ));
    }
}

#[test]
fn different_kind_copies_convert_values_and_reject_content_type_mismatches_even_when_empty() {
    check(
        "let a=new Int8Array(new Uint8Array([255,128])),b=new Uint16Array(new Int8Array([-1,7])),c=new Float16Array(new Float64Array([1+2**-11+Number.EPSILON]));a[0]===-1 && a[1]===-128 && b[0]===65535 && b[1]===7 && c[0]===1+2**-10",
    );
    check("let a=new BigInt64Array(new BigUint64Array([18446744073709551615n]));a[0]===-1n");
    for length in [0, 1] {
        exception(
            &format!("new BigInt64Array(new Uint8Array({length}))"),
            ExceptionKind::TypeError,
        );
        exception(
            &format!("new Uint8Array(new BigInt64Array({length}))"),
            ExceptionKind::TypeError,
        );
    }
    exception(
        "new BigInt64Array(new Uint8Array(9007199254740991))",
        ExceptionKind::RangeError,
    );
}

#[test]
fn iterable_construction_exhausts_before_numeric_conversion_and_never_reads_length() {
    check(
        "let log='',done=false,index=0,values=[{valueOf(){if(!done)throw 7;log+='a';return 3;}},{valueOf(){if(!done)throw 8;log+='b';return 4;}}],source={get length(){throw 9;},get [Symbol.iterator](){log+='g';return function(){log+='i';return {next(){log+='n';if(index===2){done=true;return {done:true,get value(){throw 10;}};}return {done:false,value:values[index++]};}};}}};function C(){}let N=C.bind(null);Object.defineProperty(N,'prototype',{get(){log+='p';return Uint8Array.prototype;}});let a=Reflect.construct(Uint8Array,[source],N);log==='pginnnab' && a.length===2 && a[0]===3 && a[1]===4",
    );
}

#[test]
fn iterator_and_post_exhaustion_conversion_failures_do_not_invent_iterator_close_calls() {
    check(
        "let index=0,closed=0,done=false,caught=false,source={[Symbol.iterator](){return {next(){if(index++===0)return {done:false,value:{valueOf(){if(!done)throw 8;throw 7;}}};done=true;return {done:true};},return(){closed++;throw 9;}};}};try{new Uint8Array(source);}catch(e){caught=e===7;}caught && done && closed===0",
    );
    check(
        "let closed=0,caught=false,source={[Symbol.iterator](){return {next(){throw 7;},return(){closed++;throw 8;}};}};try{new Uint8Array(source);}catch(e){caught=e===7;}caught && closed===0",
    );
    for source in [
        "({[Symbol.iterator]:7})",
        "({[Symbol.iterator](){return 7;}})",
        "({[Symbol.iterator](){return {next:7};}})",
        "({[Symbol.iterator](){return {next(){return 7;}};}})",
    ] {
        exception(
            &format!("new Uint8Array({source})"),
            ExceptionKind::TypeError,
        );
    }
}

#[test]
fn array_like_construction_reads_length_once_then_gets_and_converts_each_live_index() {
    check(
        "let log='',source={get length(){log+='l';return {valueOf(){log+='n';return 3.9;}};},get 0(){log+='a';return {valueOf(){log+='v';source[1]=9;return 7;}};},1:2};Object.setPrototypeOf(source,{2:5});let a=new Uint8Array(source);log==='lnav' && a.length===3 && a[0]===7 && a[1]===9 && a[2]===5",
    );
    check(
        "let a=new Uint8Array({length:3,1:7}),b=new Float64Array({length:2,1:7});a[0]===0 && a[1]===7 && a[2]===0 && Number.isNaN(b[0]) && b[1]===7",
    );
    exception("new BigInt64Array({length:1})", ExceptionKind::TypeError);
    exception("new Uint8Array({length:1n})", ExceptionKind::TypeError);
    check(
        "let reads=0,caught=false,source={get length(){return 2;},get 0(){throw 7;},get 1(){reads++;return 8;}};try{new Uint8Array(source);}catch(e){caught=e===7;}caught && reads===0",
    );
}

#[test]
fn factories_validate_constructor_and_mapper_before_source_hooks() {
    check(
        "let hits=0,caught=false,source={get [Symbol.iterator](){hits++;throw 7;},get length(){hits++;throw 8;}};try{Uint8Array.from.call(()=>{},source,null);}catch(e){caught=e instanceof TypeError;}caught && hits===0",
    );
    check(
        "let hits=0,caught=false,source={get [Symbol.iterator](){hits++;throw 7;}};try{Uint8Array.from(source,7);}catch(e){caught=e instanceof TypeError;}caught && hits===0",
    );
    exception("Uint8Array.of.call(()=>{},1)", ExceptionKind::TypeError);
    for source in ["null", "undefined"] {
        exception(
            &format!("Uint8Array.from({source})"),
            ExceptionKind::TypeError,
        );
    }
    check(
        "let a=Uint8Array.from('12'),b=BigInt64Array.from('12'),c=Uint8Array.from(7);a.length===2 && a[0]===1 && a[1]===2 && b[0]===1n && b[1]===2n && c.length===0",
    );
}

#[test]
fn from_exhausts_iterables_before_construction_then_maps_and_sets_in_order() {
    check(
        "let log='',index=0,context={},source={get [Symbol.iterator](){log+='g';return function(){log+='i';return {next(){log+='n';if(index===2)return {done:true};return {done:false,value:++index};}};}}};function C(n){log+='C'+n;return new Uint8Array(n);}let a=Uint8Array.from.call(C,source,function(v,i){'use strict';if(this!==context || arguments.length!==2)throw 7;log+='m'+i;return {valueOf(){log+='v'+i;return v+1;}};},context);log==='ginnnC2m0v0m1v1' && a[0]===2 && a[1]===3",
    );
    check(
        "let log='',source={get [Symbol.iterator](){log+='i';return null;},get length(){log+='l';return 2;},get 0(){log+='a';return 1;},get 1(){log+='b';return 2;}};function C(n){log+='C'+n;return new Uint8Array(n);}let a=Uint8Array.from.call(C,source,(v,i)=>{log+='m'+i;return v+i;});log==='ilC2am0bm1' && a[0]===1 && a[1]===3",
    );
    check(
        "let done=false,closed=0,caught=false,index=0,source={[Symbol.iterator](){return {next(){if(index++===0)return {value:1,done:false};done=true;return {done:true};},return(){closed++;}};}};try{Uint8Array.from(source,()=>{if(!done)throw 8;throw 7;});}catch(e){caught=e===7;}caught && done && closed===0",
    );
}

#[test]
fn from_and_of_are_generic_construct_once_and_accept_larger_or_different_kind_results() {
    check(
        "let target=new Uint8Array([9,9,9]),n,argc,newTarget,calls=0;function C(length){calls++;n=length;argc=arguments.length;newTarget=new.target;return target;}Object.defineProperty(C,Symbol.species,{get(){throw 7;}});let a=Uint16Array.of.call(C,257,2);a===target && a[0]===1 && a[1]===2 && a[2]===9 && n===2 && argc===1 && newTarget===C && calls===1",
    );
    check(
        "let target=new BigInt64Array(2);function C(){return target;}let a=Uint8Array.from.call(C,['1','2']);a===target && a[0]===1n && a[1]===2n && Uint8Array.of.call(C,3n,4n)===target && target[0]===3n && target[1]===4n",
    );
    for (name, _, bigint) in KINDS {
        let one = if bigint { "1n" } else { "1" };
        let two = if bigint { "2n" } else { "2" };
        check(&format!(
            "let a={name}.of({one},{two}),b={name}.from([{one},{two}]);a instanceof {name} && b instanceof {name} && a.length===2 && b.length===2 && a[0]==={one} && a[1]==={two} && b[0]==={one} && b[1]==={two}"
        ));
    }
}

#[test]
fn factory_results_must_be_live_branded_and_at_least_the_requested_length() {
    for result in [
        "{}",
        "[]",
        "new DataView(new ArrayBuffer(2))",
        "Uint8Array.prototype",
        "Object.create(new Uint8Array(2))",
        "new Uint8Array(1)",
        "(()=>{let a=new Uint8Array(2);a.buffer.transfer();return a;})()",
        "(()=>{let b=new ArrayBuffer(2,{maxByteLength:4}),a=new Uint8Array(b,0,2);b.resize(1);return a;})()",
    ] {
        check(&format!(
            "let hits=0,caught=false;function C(){{return {result};}}try{{Uint8Array.from.call(C,[1,2],v=>{{hits++;return v;}});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
        ));
    }
    check(
        "let target=new Uint8Array(0);target.buffer.transfer();function C(){return target;}let caught=false;try{Uint8Array.of.call(C);}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn factory_mapping_and_value_conversion_can_detach_or_resize_custom_results() {
    check(
        "let result,moved,log='';function C(n){result=new Uint8Array(n);return result;}let a=Uint8Array.from.call(C,[1,2],(v,i)=>{log+='m'+i;if(i===0)moved=result.buffer.transfer();return {valueOf(){log+='v'+i;return v;}};});a===result && a.length===0 && a.buffer.detached && log==='m0v0m1v1' && new DataView(moved).getUint16(0)===0",
    );
    check(
        "let result,log='';function C(n){result=new Uint8Array(n);return result;}let a=Uint8Array.of.call(C,{valueOf(){log+='a';result.buffer.transfer();return 1;}},{valueOf(){log+='b';return 2;}});a===result && a.length===0 && log==='ab'",
    );
    check(
        "let result;function C(n){result=new Uint8Array(new ArrayBuffer(n,{maxByteLength:4}));return result;}let a=Uint8Array.from.call(C,{length:2,get 0(){result.buffer.resize(0);return {valueOf(){return 7;}};},get 1(){result.buffer.resize(2);return 9;}});a===result && a.length===2 && a[0]===0 && a[1]===9",
    );
}

#[test]
fn intrinsic_and_live_state_matrix() {
    let Value::String(output) = Realm::default()
        .eval(
            r"let rows=[];for(let C of [Int8Array,Uint8Array,Uint8ClampedArray,Int16Array,Uint16Array,Int32Array,Uint32Array,BigInt64Array,BigUint64Array,Float16Array,Float32Array,Float64Array]){let a=new C(2);rows.push(C.name+' '+[C.length,C.BYTES_PER_ELEMENT,a.length,a.byteLength,Object.prototype.toString.call(a)].join(','));}let buffer=new ArrayBuffer(6,{maxByteLength:10}),fixed=new Uint16Array(buffer,2,2),auto=new Uint16Array(buffer,2);function state(name,a){rows.push(name+' '+[a.length,a.byteLength,a.byteOffset,a.buffer===buffer,Object.prototype.toString.call(a)].join(','));}state('fixed',fixed);state('auto',auto);buffer.resize(5);state('fixed-outside',fixed);state('auto-small',auto);buffer.resize(2);state('auto-empty',auto);buffer.resize(1);state('auto-outside',auto);buffer.resize(8);state('fixed-regrown',fixed);state('auto-regrown',auto);buffer.transfer();state('fixed-detached',fixed);state('auto-detached',auto);rows.join('\n')",
        )
        .unwrap()
    else {
        panic!("TypedArray metadata and state matrix must be a string")
    };
    insta::assert_snapshot!(output.to_utf8().unwrap(), @"
    Int8Array 3,1,2,2,[object Int8Array]
    Uint8Array 3,1,2,2,[object Uint8Array]
    Uint8ClampedArray 3,1,2,2,[object Uint8ClampedArray]
    Int16Array 3,2,2,4,[object Int16Array]
    Uint16Array 3,2,2,4,[object Uint16Array]
    Int32Array 3,4,2,8,[object Int32Array]
    Uint32Array 3,4,2,8,[object Uint32Array]
    BigInt64Array 3,8,2,16,[object BigInt64Array]
    BigUint64Array 3,8,2,16,[object BigUint64Array]
    Float16Array 3,2,2,4,[object Float16Array]
    Float32Array 3,4,2,8,[object Float32Array]
    Float64Array 3,8,2,16,[object Float64Array]
    fixed 2,4,2,true,[object Uint16Array]
    auto 2,4,2,true,[object Uint16Array]
    fixed-outside 0,0,0,true,[object Uint16Array]
    auto-small 1,2,2,true,[object Uint16Array]
    auto-empty 0,0,2,true,[object Uint16Array]
    auto-outside 0,0,0,true,[object Uint16Array]
    fixed-regrown 2,4,2,true,[object Uint16Array]
    auto-regrown 3,6,2,true,[object Uint16Array]
    fixed-detached 0,0,0,true,[object Uint16Array]
    auto-detached 0,0,0,true,[object Uint16Array]
    ");
}

#[test]
fn intrinsic_graph_and_bytes_per_element_descriptors_are_complete() {
    for (name, width, _) in KINDS {
        check(&format!(
            "let C={name},T=Object.getPrototypeOf(C),shared=T.prototype,p=C.prototype,g=Object.getOwnPropertyDescriptor(globalThis,'{name}'),d=Object.getOwnPropertyDescriptor(C,'prototype'),b=Object.getOwnPropertyDescriptor(C,'BYTES_PER_ELEMENT'),pb=Object.getOwnPropertyDescriptor(p,'BYTES_PER_ELEMENT'),c=Object.getOwnPropertyDescriptor(p,'constructor');T.name==='TypedArray' && T.length===0 && typeof TypedArray==='undefined' && Object.getPrototypeOf(T)===Function.prototype && Object.getPrototypeOf(shared)===Object.prototype && Object.getPrototypeOf(p)===shared && shared.constructor===T && C.name==='{name}' && C.length===3 && g.value===C && g.writable && !g.enumerable && g.configurable && d.value===p && !d.writable && !d.enumerable && !d.configurable && b.value==={width} && !b.writable && !b.enumerable && !b.configurable && pb.value==={width} && !pb.writable && !pb.enumerable && !pb.configurable && c.value===C && c.writable && !c.enumerable && c.configurable && C.from===T.from && C.of===T.of && C[Symbol.species]===C && !Object.hasOwn(C,Symbol.species)"
        ));
    }
    check(
        "let T=Object.getPrototypeOf(Uint8Array),p=T.prototype,d=Object.getOwnPropertyDescriptor(T,Symbol.species),receiver={};Reflect.ownKeys(p).length===38 && !Object.hasOwn(p,'BYTES_PER_ELEMENT') && !Object.hasOwn(p,Symbol.unscopables) && d.get.name==='get [Symbol.species]' && d.get.length===0 && d.set===undefined && !d.enumerable && d.configurable && d.get.call(receiver)===receiver && d.get.call(null)===null",
    );
    exception(
        "Object.getPrototypeOf(Uint8Array)()",
        ExceptionKind::TypeError,
    );
    exception(
        "new (Object.getPrototypeOf(Uint8Array))",
        ExceptionKind::TypeError,
    );
}

#[test]
fn iterator_alias_and_original_array_to_string_identity_are_preserved() {
    check(
        "let p=Object.getPrototypeOf(Uint8Array.prototype),original=Array.prototype.toString;Array.prototype.toString=function(){throw 7;};let a=new Uint8Array([1,2]);a.join=()=>7;p.toString===original && p[Symbol.iterator]===p.values && p.values.name==='values' && p.values.length===0 && a.toString()===7",
    );
    check(
        "let T=Object.getPrototypeOf(Uint8Array),p=T.prototype;for(let [name,length] of [['from',1],['of',0]]){let d=Object.getOwnPropertyDescriptor(T,name);if(d.value.name!==name || d.value.length!==length || !d.writable || d.enumerable || !d.configurable || Object.hasOwn(d.value,'prototype'))throw 7;}for(let name of ['keys','values','entries']){let d=Object.getOwnPropertyDescriptor(p,name);if(d.value.name!==name || d.value.length!==0 || !d.writable || d.enumerable || !d.configurable || Object.hasOwn(d.value,'prototype'))throw 8;}true",
    );
    check(
        "let p=Object.getPrototypeOf(Uint8Array.prototype),lengths={at:1,copyWithin:2,every:1,fill:1,filter:1,find:1,findIndex:1,findLast:1,findLastIndex:1,forEach:1,includes:1,indexOf:1,join:1,lastIndexOf:1,map:1,reduce:1,reduceRight:1,reverse:0,set:1,slice:2,some:1,sort:1,subarray:2,toLocaleString:0,toReversed:0,toSorted:1,with:2};for(let name of Object.keys(lengths)){let d=Object.getOwnPropertyDescriptor(p,name);if(d.value.name!==name || d.value.length!==lengths[name] || !d.writable || d.enumerable || !d.configurable || Object.hasOwn(d.value,'prototype'))throw 7;}for(let [object,name,length] of [[Uint8Array,'fromBase64',1],[Uint8Array,'fromHex',1],[Uint8Array.prototype,'setFromBase64',1],[Uint8Array.prototype,'setFromHex',1],[Uint8Array.prototype,'toBase64',0],[Uint8Array.prototype,'toHex',0]]){let d=Object.getOwnPropertyDescriptor(object,name);if(d.value.name!==name || d.value.length!==length || !d.writable || d.enumerable || !d.configurable || Object.hasOwn(d.value,'prototype'))throw 8;}true",
    );
}

#[test]
fn state_getters_are_branded_but_the_tag_getter_is_generic() {
    for receiver in [
        "undefined",
        "null",
        "3",
        "'x'",
        "1n",
        "Symbol()",
        "{}",
        "Uint8Array.prototype",
        "Object.create(new Uint8Array(2))",
    ] {
        for name in ["buffer", "byteLength", "byteOffset", "length"] {
            exception(
                &format!(
                    "Object.getOwnPropertyDescriptor(Object.getPrototypeOf(Uint8Array.prototype),'{name}').get.call({receiver})"
                ),
                ExceptionKind::TypeError,
            );
        }
        check(&format!(
            "let tag=Object.getOwnPropertyDescriptor(Object.getPrototypeOf(Uint8Array.prototype),Symbol.toStringTag).get;tag.call({receiver})===undefined"
        ));
    }
    check(
        "let p=Object.getPrototypeOf(Uint8Array.prototype);for(let name of ['buffer','byteLength','byteOffset','length']){let d=Object.getOwnPropertyDescriptor(p,name);if(d.get.name!=='get '+name || d.get.length!==0 || d.set!==undefined || d.enumerable || !d.configurable || Object.hasOwn(d.get,'prototype'))throw 7;}let d=Object.getOwnPropertyDescriptor(p,Symbol.toStringTag);d.get.name==='get [Symbol.toStringTag]' && d.get.length===0 && d.set===undefined && !d.enumerable && d.configurable && !Object.hasOwn(d.get,'prototype')",
    );
}

#[test]
fn intrinsic_iterators_validate_at_creation_then_follow_live_view_bounds() {
    for method in ["keys", "values", "entries"] {
        check(&format!(
            "let b=new ArrayBuffer(4,{{maxByteLength:8}}),a=new Uint8Array(b,1,3),caught=false;b.resize(2);Object.defineProperty(a,'length',{{get(){{throw 7;}}}});try{{a.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
        check(&format!(
            "let a=new Uint8Array(2),caught=false;a.buffer.transfer();try{{a.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    check(
        "let b=new ArrayBuffer(2,{maxByteLength:6}),a=new Uint8Array(b),iterator=a.values();a[0]=7;a[1]=8;let first=iterator.next();b.resize(3);a[2]=9;let second=iterator.next(),third=iterator.next(),end=iterator.next();b.resize(6);first.value===7 && second.value===8 && third.value===9 && end.done && iterator.next().done",
    );
}

#[test]
fn subclasses_initialize_private_fields_after_copy_and_factories_retain_their_constructor() {
    check(
        "let args,target;class C extends Uint16Array{#first=this[0];#length=this.length;constructor(...xs){super(...xs);args=xs;target=new.target;}read(){return this.#first+this.#length;}static get [Symbol.species](){throw 7;}}let a=new C([7,9]),b=C.from([3,4]),c=C.of(1,2);a instanceof C && a.read()===9 && b instanceof C && b.read()===2 && b[0]===3 && c instanceof C && c.read()===2 && c[0]===1 && args.length===1 && args[0]===2 && target===C",
    );
}

#[test]
fn copied_and_subclassed_views_keep_buffers_and_captured_values_alive_through_collection() {
    let mut realm = Realm::default();
    realm
        .eval(
            "let result,copy,read;(()=>{let capture={value:7};class C extends Uint8Array{#capture=capture;read(){return this.#capture.value;}}let b=new ArrayBuffer(6,{maxByteLength:10});result=new C(b,1,3);result[0]=8;result[2]=9;copy=new Uint8Array(result);read=result.read;})();",
        )
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(
            "read.call(result)===7 && result[0]===8 && result[2]===9 && copy[0]===8 && copy[2]===9 && copy.buffer!==result.buffer && result.buffer.byteLength===6",
        ),
        Ok(Value::Boolean(true))
    );
    realm
        .eval("result.buffer.resize(0);result.buffer.resize(6)")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("read.call(result)===7 && result.length===3 && result[0]===0 && copy[0]===8"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn iterable_lists_do_not_use_argument_quotas_and_opted_in_host_limits_remain_uncatchable() {
    let mut realm = Realm::new(Limits {
        max_arguments: Some(1),
        ..Limits::default()
    });
    assert_eq!(
        realm.eval(
            "let a=new Uint8Array([1,2,3,4]),b=Uint8Array.from([5,6,7]);a.length===4 && a[3]===4 && b.length===3 && b[2]===7",
        ),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_bigint_bits: Some(8),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert_eq!(
        realm.eval(
            "let bytes=new Uint8Array(8);for(let i=0;i<8;i++)bytes[i]=255;let source=new BigUint64Array(bytes.buffer),copy=new BigUint64Array(source),raw=new Uint8Array(copy.buffer);raw.length===8 && raw[0]===255 && raw[7]===255 && copy.buffer!==source.buffer",
        ),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval("try{new BigInt64Array(source);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert!(matches!(
        realm.eval("try{new BigUint64Array(['256']);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(1_000),
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{new Uint8Array({length:100000});}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}

#[test]
fn deferred_algorithms_report_host_unsupported_without_entering_language_handlers() {
    let mut expressions = Vec::new();
    for name in [
        "copyWithin",
        "fill",
        "reverse",
        "set",
        "slice",
        "sort",
        "subarray",
        "toReversed",
        "toSorted",
        "with",
    ] {
        expressions.push(format!("new Uint8Array(1).{name}()"));
    }
    expressions.extend([
        "Uint8Array.fromBase64('')".into(),
        "Uint8Array.fromHex('')".into(),
        "new Uint8Array(1).setFromBase64('')".into(),
        "new Uint8Array(1).setFromHex('')".into(),
        "new Uint8Array(1).toBase64()".into(),
        "new Uint8Array(1).toHex()".into(),
    ]);
    for expression in expressions {
        let mut realm = Realm::default();
        realm.eval("let flag=0").unwrap();
        let result = realm.eval(&format!(
            "try{{{expression};}}catch{{flag=1;}}finally{{flag=2;}}"
        ));
        assert!(
            matches!(&result, Err(Error::Unsupported { .. })),
            "{expression}: {result:?}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
