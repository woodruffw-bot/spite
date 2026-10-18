//! TypedArray raw moves, numeric mutation, and set sources (23.2.3.6/.9/.25/.26).

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

const KINDS: [(&str, bool); 12] = [
    ("Int8Array", false),
    ("Uint8Array", false),
    ("Uint8ClampedArray", false),
    ("Int16Array", false),
    ("Uint16Array", false),
    ("Int32Array", false),
    ("Uint32Array", false),
    ("BigInt64Array", true),
    ("BigUint64Array", true),
    ("Float16Array", false),
    ("Float32Array", false),
    ("Float64Array", false),
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
fn all_twelve_kinds_mutate_in_place_with_the_specified_return_values() {
    for (name, bigint) in KINDS {
        let values = if bigint {
            "1n,2n,3n,4n,5n"
        } else {
            "1,2,3,4,5"
        };
        let two = if bigint { "2n" } else { "2" };
        let source = if bigint { "3n,1n" } else { "3,1" };
        check(&format!(
            "let a={name}.of({values});a.copyWithin(1,3)===a && a.join()==='1,4,5,4,5' && a.fill({two},1,3)===a && a.join()==='1,2,2,4,5' && a.reverse()===a && a.join()==='5,4,2,2,1' && a.set({name}.of({source}),2)===undefined && a.join()==='5,4,3,1,1'"
        ));
        check(&format!(
            "let a=new {name}(0);a.copyWithin(0,0)===a && a.fill({two})===a && a.reverse()===a && a.set(new {name}(0))===undefined"
        ));
    }
}

#[test]
fn mutation_uses_internal_state_and_ignores_public_length_constructor_and_iterator() {
    check(
        "let a=Uint16Array.of(1,2,3,4),source=Uint8Array.of(7,8);for(let object of [a,source])for(let key of ['length','buffer','byteLength','byteOffset','BYTES_PER_ELEMENT','constructor',Symbol.iterator])Object.defineProperty(object,key,{get(){throw 7;}});a.copyWithin(1,2)===a && a[0]===1 && a[1]===3 && a[2]===4 && a.fill(9,2)===a && a[2]===9 && a[3]===9 && a.reverse()===a && a[0]===9 && a[3]===1 && a.set(source,1)===undefined && a[0]===9 && a[1]===7 && a[2]===8 && a[3]===1",
    );
}

#[test]
fn receivers_require_their_own_typed_array_brand_before_argument_conversion() {
    for method in ["copyWithin", "fill", "reverse", "set"] {
        for receiver in [
            "null",
            "{}",
            "new DataView(new ArrayBuffer(4))",
            "Object.create(new Uint8Array(4))",
        ] {
            exception(
                &format!(
                    "Uint8Array.prototype.{method}.call({receiver},{{valueOf(){{throw 7;}},get length(){{throw 8;}}}},{{valueOf(){{throw 9;}}}})"
                ),
                ExceptionKind::TypeError,
            );
        }
    }
    for method in ["copyWithin", "fill", "reverse"] {
        for invalidation in ["a.buffer.transfer()", "buffer.resize(1)"] {
            check(&format!(
                "let buffer=new ArrayBuffer(4,{{maxByteLength:8}}),a=new Uint8Array(buffer,0,4),hits=0,caught=false;{invalidation};try{{a.{method}({{valueOf(){{hits++;throw 7;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && hits===0"
            ));
        }
    }
}

#[test]
fn copy_within_converts_and_clamps_relative_indices_in_order() {
    for (arguments, expected) in [
        ("1,0,3", "1,1,2,3,5"),
        ("-2,0", "1,2,3,1,2"),
        ("-2,0,undefined", "1,2,3,1,2"),
        ("Infinity,0", "1,2,3,4,5"),
        ("-Infinity,2,Infinity", "3,4,5,4,5"),
        ("NaN,1.9,3.9", "2,3,3,4,5"),
        ("0,-Infinity,-Infinity", "1,2,3,4,5"),
        ("undefined,undefined,undefined", "1,2,3,4,5"),
    ] {
        check(&format!(
            "let a=Uint8Array.of(1,2,3,4,5);a.copyWithin({arguments})===a && a.join()==='{expected}'"
        ));
    }
    check(
        "let a=Uint8Array.of(1,2,3),log='';a.copyWithin({valueOf(){log+='t';return 1;}},{valueOf(){log+='s';return 0;}},{valueOf(){log+='e';return 2;}});log==='tse' && a.join()==='1,1,2'",
    );
    for arguments in ["Symbol(),0", "0,Symbol()", "0,0,Symbol()", "1n,0"] {
        exception(
            &format!("new Uint8Array(0).copyWithin({arguments})"),
            ExceptionKind::TypeError,
        );
    }
}

#[test]
fn copy_within_only_revalidates_when_the_original_count_is_positive() {
    check(
        "let a=new Uint8Array(0),log='';let result=a.copyWithin({valueOf(){log+='t';a.buffer.transfer();return 0;}},{valueOf(){log+='s';return 0;}},{valueOf(){log+='e';return 0;}});result===a && log==='tse' && a.length===0",
    );
    check(
        "let buffer=new ArrayBuffer(3,{maxByteLength:6}),a=new Uint8Array(buffer,0,3);let result=a.copyWithin(3,0,{valueOf(){buffer.resize(0);return 3;}});result===a && a.length===0",
    );
    check(
        "let a=Uint8Array.of(1,2,3),log='',caught=false;try{a.copyWithin({valueOf(){log+='t';a.buffer.transfer();return 1;}},{valueOf(){log+='s';return 0;}},{valueOf(){log+='e';return 2;}});}catch(e){caught=e instanceof TypeError;}caught && log==='tse'",
    );
    assert_eq!(
        Realm::default().eval("let a=Uint8Array.of(1,2,3);a.copyWithin({valueOf(){a.buffer.transfer();return 1;}},{valueOf(){throw 7;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    for setup in [
        "let buffer=new ArrayBuffer(6,{maxByteLength:8}),a=new Uint8Array(buffer,0,6);",
        "let buffer=new ArrayBuffer(8,{maxByteLength:10}),a=new Uint8Array(buffer,2);",
    ] {
        exception(
            &format!("{setup}a.copyWithin({{valueOf(){{buffer.resize(1);return 4;}}}},2,6)"),
            ExceptionKind::TypeError,
        );
    }
}

#[test]
fn copy_within_clips_a_live_auto_view_and_never_copies_growth_beyond_the_original_range() {
    check(
        "let buffer=new ArrayBuffer(6,{maxByteLength:8}),a=new Uint8Array(buffer);a.set([1,2,3,4,5,6]);let result=a.copyWithin(1,3,{valueOf(){buffer.resize(5);return 6;}});result===a && a.join()==='1,4,5,4,5'",
    );
    check(
        "let buffer=new ArrayBuffer(8,{maxByteLength:10}),a=new Uint8Array(buffer,2);let result=a.copyWithin({valueOf(){buffer.resize(2);return 4;}},2,6);result===a && a.length===0 && buffer.byteLength===2",
    );
    check(
        "let buffer=new ArrayBuffer(3,{maxByteLength:6}),a=new Uint8Array(buffer);a.set([1,2,3]);a.copyWithin(1,0,{valueOf(){buffer.resize(6);a[5]=7;return Infinity;}});a.join()==='1,1,2,0,0,7'",
    );
}

#[test]
fn raw_copy_within_and_same_kind_set_preserve_nan_payloads_in_both_overlap_directions() {
    for name in ["Float16Array", "Float32Array", "Float64Array"] {
        for (operation, expected_index) in [
            ("a.copyWithin(1,0,2)", "i<size?i:i-size"),
            ("a.copyWithin(0,1,3)", "i<2*size?i+size:i"),
            ("a.set(new A(buffer,size,2),1)", "i<size?i:i-size"),
            ("a.set(new A(buffer,2*size,2),0)", "i<2*size?i+size:i"),
        ] {
            check(&format!(
                "let A={name},size=A.BYTES_PER_ELEMENT,little=new Uint8Array(Uint16Array.of(1).buffer)[0]===1,buffer=new ArrayBuffer(5*size),d=new DataView(buffer),a=new A(buffer,size,3),before=[];for(let i=0;i<5*size;i++)d.setUint8(i,90);for(let i=0;i<3*size;i++)d.setUint8(size+i,i%size===(little?0:size-1)?17*(1+Math.floor(i/size)):255);for(let i=0;i<3*size;i++)before.push(d.getUint8(size+i));let valid=Number.isNaN(a[0]) && Number.isNaN(a[1]) && Number.isNaN(a[2]);{operation};for(let i=0;i<3*size;i++)valid=valid && d.getUint8(size+i)===before[{expected_index}];for(let i=0;i<size;i++)valid=valid && d.getUint8(i)===90 && d.getUint8(4*size+i)===90;valid"
            ));
        }
    }
}

#[test]
fn distinct_same_kind_set_preserves_raw_bytes_offsets_and_unwritten_target_regions() {
    for name in ["Float16Array", "Float32Array", "Float64Array"] {
        check(&format!(
            "let A={name},size=A.BYTES_PER_ELEMENT,little=new Uint8Array(Uint16Array.of(1).buffer)[0]===1,sourceBuffer=new ArrayBuffer(4*size),s=new DataView(sourceBuffer),source=new A(sourceBuffer,size,3),targetBuffer=new ArrayBuffer(6*size),d=new DataView(targetBuffer),target=new A(targetBuffer,size,4),valid=true;for(let i=0;i<3*size;i++)s.setUint8(size+i,i%size===(little?0:size-1)?17*(1+Math.floor(i/size)):255);for(let i=0;i<6*size;i++)d.setUint8(i,90);let result=target.set(source,1);for(let i=0;i<3*size;i++)valid=valid && d.getUint8(2*size+i)===s.getUint8(size+i);for(let i=0;i<2*size;i++)valid=valid && d.getUint8(i)===90;for(let i=5*size;i<6*size;i++)valid=valid && d.getUint8(i)===90;s.setUint8(size+(little?0:size-1),99);valid && result===undefined && d.getUint8(2*size+(little?0:size-1))===17"
        ));
    }
}

#[test]
fn raw_moves_cross_page_boundaries_and_zero_source_bytes_overwrite_old_targets() {
    check(
        "let a=new Uint8Array(4110);a[4088]=98;a[4089]=1;a[4090]=2;a[4091]=3;a[4093]=4;a[4094]=5;a[4095]=6;a[4097]=7;a[4098]=8;a[4099]=9;a[4101]=99;a[4102]=99;a[4103]=99;a[4104]=99;a[4105]=97;a.copyWithin(4093,4089,4101);let first=new Uint8Array(a.buffer,4093,12).join()==='1,2,3,0,4,5,6,0,7,8,9,0';a.copyWithin(4089,4093,4105);first && new Uint8Array(a.buffer,4089,12).join()==='1,2,3,0,4,5,6,0,7,8,9,0' && a[4088]===98 && a[4105]===97",
    );
}

#[test]
fn fill_converts_its_value_once_before_bounds_even_for_empty_views() {
    for (name, value, expected) in [("Uint8Array", "7", "7"), ("BigInt64Array", "'7'", "7n")] {
        check(&format!(
            "let a=new {name}(3),log='',hits=0;let result=a.fill({{[Symbol.toPrimitive](hint){{log+='v'+hint;hits++;return {value};}}}},{{valueOf(){{log+='s';return 1;}}}},{{valueOf(){{log+='e';return 3;}}}});result===a && log==='vnumberse' && hits===1 && a[1]==={expected} && a[2]==={expected}"
        ));
        check(&format!(
            "let a=new {name}(0),log='';a.fill({{valueOf(){{log+='v';return {value};}}}},{{valueOf(){{log+='s';return 0;}}}},{{valueOf(){{log+='e';return 0;}}}})===a && log==='vse'"
        ));
    }
    exception("new Uint8Array(0).fill(1n)", ExceptionKind::TypeError);
    exception("new BigInt64Array(0).fill(1)", ExceptionKind::TypeError);
    for arguments in ["1,Symbol()", "1,0,Symbol()"] {
        exception(
            &format!("new Uint8Array(0).fill({arguments})"),
            ExceptionKind::TypeError,
        );
    }
    assert_eq!(
        Realm::default()
            .eval("new Uint8Array(0).fill({valueOf(){throw 7;}},{valueOf(){throw 8;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
}

#[test]
fn fill_clamps_bounds_and_revalidates_unconditionally_after_all_conversions() {
    for (arguments, expected) in [
        ("9,-2", "1,2,3,9,9"),
        ("9,Infinity", "1,2,3,4,5"),
        ("9,-Infinity,2.9", "9,9,3,4,5"),
        ("9,NaN,Infinity", "9,9,9,9,9"),
        ("9,0,-Infinity", "1,2,3,4,5"),
        ("9,undefined,undefined", "9,9,9,9,9"),
    ] {
        check(&format!(
            "let a=Uint8Array.of(1,2,3,4,5);a.fill({arguments})===a && a.join()==='{expected}'"
        ));
    }
    for length in [0, 3] {
        check(&format!(
            "let a=new Uint8Array({length}),log='',caught=false;try{{a.fill({{valueOf(){{log+='v';a.buffer.transfer();return 7;}}}},{{valueOf(){{log+='s';return 3;}}}},{{valueOf(){{log+='e';return 0;}}}});}}catch(e){{caught=e instanceof TypeError;}}caught && log==='vse'"
        ));
    }
    exception(
        "let buffer=new ArrayBuffer(3,{maxByteLength:6}),a=new Uint8Array(buffer,0,3);a.fill(7,3,{valueOf(){buffer.resize(0);return 0;}})",
        ExceptionKind::TypeError,
    );
}

#[test]
fn fill_clips_shrink_preserves_original_growth_bounds_and_accepts_recovery() {
    check(
        "let buffer=new ArrayBuffer(4,{maxByteLength:8}),a=new Uint8Array(buffer);a.set([1,2,3,4]);a.fill(7,1,{valueOf(){buffer.resize(2);return 4;}})===a && a.join()==='1,7'",
    );
    check(
        "let buffer=new ArrayBuffer(4,{maxByteLength:8}),a=new Uint8Array(buffer);a.set([1,2,3,4]);a.fill({valueOf(){buffer.resize(8);a[7]=99;return 7;}},-1);a.join()==='1,2,3,7,0,0,0,99'",
    );
    check(
        "let buffer=new ArrayBuffer(3,{maxByteLength:6}),a=new Uint8Array(buffer,0,3);a.set([1,2,3]);a.fill({valueOf(){buffer.resize(1);return 7;}},{valueOf(){buffer.resize(3);return 1;}})===a && a.join()==='1,7,7'",
    );
    exception(
        "let buffer=new ArrayBuffer(4,{maxByteLength:8}),a=new Uint8Array(buffer,0,4);a.fill(7,1,{valueOf(){buffer.resize(2);return 4;}})",
        ExceptionKind::TypeError,
    );
}

#[test]
fn reverse_preserves_signed_zero_and_uses_numeric_nan_writes_without_touching_the_middle() {
    check(
        "let a=Float64Array.of(-0,2,0);a.reverse()===a && Object.is(a[0],0) && a[1]===2 && Object.is(a[2],-0)",
    );
    for (name, setter) in [
        ("Float16Array", "setFloat16"),
        ("Float32Array", "setFloat32"),
        ("Float64Array", "setFloat64"),
    ] {
        check(&format!(
            "let A={name},size=A.BYTES_PER_ELEMENT,little=new Uint8Array(Uint16Array.of(1).buffer)[0]===1,buffer=new ArrayBuffer(3*size),d=new DataView(buffer),a=new A(buffer),expected=new DataView(new ArrayBuffer(size)),valid=true;for(let i=0;i<3*size;i++)d.setUint8(i,255);expected.{setter}(0,NaN,little);a.reverse();for(let i=0;i<size;i++)valid=valid && d.getUint8(i)===expected.getUint8(i) && d.getUint8(2*size+i)===expected.getUint8(i) && d.getUint8(size+i)===255;valid && Number.isNaN(a[0]) && Number.isNaN(a[1]) && Number.isNaN(a[2])"
        ));
    }
}

#[test]
fn set_offset_precedes_buffer_witnesses_but_follows_the_brand_check() {
    for offset in ["-1", "-1.9", "-Infinity"] {
        exception(
            &format!(
                "let a=new Uint8Array(1);a.buffer.transfer();a.set({{get length(){{throw 7;}}}},{offset})"
            ),
            ExceptionKind::RangeError,
        );
    }
    assert_eq!(
        Realm::default()
            .eval("let a=new Uint8Array(1);a.buffer.transfer();a.set(null,{valueOf(){throw 7;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    check(
        "let buffer=new ArrayBuffer(4,{maxByteLength:8}),a=new Uint8Array(buffer,0,4),log='';buffer.resize(1);let result=a.set({get length(){log+='l';return 1;},0:7},{valueOf(){log+='o';buffer.resize(4);return 0;}});result===undefined && log==='ol' && a[0]===7 && a.length===4",
    );
    check(
        "let a=new Uint8Array(3),log='',caught=false;try{a.set({get length(){log+='l';throw 7;}},{valueOf(){log+='o';a.buffer.transfer();return Infinity;}});}catch(e){caught=e instanceof TypeError;}caught && log==='o'",
    );
    for offset in ["undefined", "NaN", "-0", "-0.5"] {
        check(&format!(
            "let a=new Uint8Array(2);a.set([7],{offset})===undefined && a[0]===7 && a[1]===0"
        ));
    }
    check("let a=new Uint8Array(3);a.set([7],1.9)===undefined && a.join()==='0,7,0'");
    exception("new Uint8Array(0).set([],1n)", ExceptionKind::TypeError);
    exception(
        "new Uint8Array(0).set([],Symbol())",
        ExceptionKind::TypeError,
    );
}

#[test]
fn set_array_like_length_and_typed_source_validation_precede_range_and_content_errors() {
    exception(
        "new Uint8Array(0).set(null,Infinity)",
        ExceptionKind::TypeError,
    );
    assert_eq!(
        Realm::default().eval("new Uint8Array(0).set({get length(){throw 7;}},Infinity)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    check(
        "let log='',caught=false;try{new Uint8Array(1).set({get length(){log+='l';return 2;},get 0(){log+='g';throw 7;}},0);}catch(e){caught=e instanceof RangeError;}caught && log==='l'",
    );
    check(
        "let log='',caught=false;try{new Uint8Array(0).set({get length(){log+='l';return 0;}},Infinity);}catch(e){caught=e instanceof RangeError;}caught && log==='l'",
    );
    exception(
        "let source=new Uint8Array(0);source.buffer.transfer();new Uint8Array(0).set(source,Infinity)",
        ExceptionKind::TypeError,
    );
    exception(
        "let buffer=new ArrayBuffer(4,{maxByteLength:8}),source=new Uint8Array(buffer,0,4);buffer.resize(1);new Uint8Array(4).set(source,Infinity)",
        ExceptionKind::TypeError,
    );
    exception(
        "new Uint8Array(0).set(new BigInt64Array(1))",
        ExceptionKind::RangeError,
    );
    exception(
        "new Uint8Array(1).set(new BigInt64Array(1))",
        ExceptionKind::TypeError,
    );
    exception(
        "new Uint8Array(0).set(new BigInt64Array(0),Infinity)",
        ExceptionKind::RangeError,
    );
    exception(
        "new Uint8Array(0).set(new BigInt64Array(0))",
        ExceptionKind::TypeError,
    );
    exception(
        "new BigInt64Array(0).set(new Uint8Array(0))",
        ExceptionKind::TypeError,
    );
    check(
        "let target=new Uint8Array(4),buffer=new ArrayBuffer(4,{maxByteLength:8}),source=new Uint8Array(buffer);source[0]=7;source[1]=8;target.set(source,{valueOf(){buffer.resize(2);return 1;}})===undefined && target.join()==='0,7,8,0'",
    );
}

#[test]
fn same_buffer_typed_set_snapshots_source_values_even_when_element_widths_differ() {
    check(
        "let buffer=new ArrayBuffer(8),source=new Uint8Array(buffer,0,4),target=new Uint16Array(buffer);source[0]=1;source[1]=2;source[2]=3;source[3]=4;target.set(source)===undefined && target[0]===1 && target[1]===2 && target[2]===3 && target[3]===4",
    );
    check(
        "let buffer=new ArrayBuffer(8),source=new Uint16Array(buffer),target=new Uint8Array(buffer,2,4);source[0]=257;source[1]=514;source[2]=771;source[3]=1028;target.set(source)===undefined && target.join()==='1,2,3,4'",
    );
    check(
        "let buffer=new ArrayBuffer(32),source=new Uint8Array(buffer,24,2),target=new Float64Array(buffer,0,2);source[0]=255;source[1]=128;target.set(source)===undefined && target[0]===255 && target[1]===128 && source[0]===255 && source[1]===128",
    );
    check(
        "let a=Uint8Array.of(1,2,3,4),source=new Uint8Array(a.buffer,0,3);a.set(source,1)===undefined && a.join()==='1,1,2,3'",
    );
}

#[test]
fn set_converts_between_numeric_kinds_including_clamping_and_bigint_wrapping() {
    check(
        "let a=new Uint8ClampedArray(7);a.set(Float64Array.of(-1,0.5,1.5,2.5,254.5,255.5,NaN));a.join()==='0,0,2,2,254,255,0'",
    );
    check(
        "let a=new BigUint64Array(2);a.set(BigInt64Array.of(-1n,-2n));a[0]===18446744073709551615n && a[1]===18446744073709551614n",
    );
    check(
        "let a=new BigInt64Array(2);a.set(BigUint64Array.of(18446744073709551615n,9223372036854775808n));a[0]===-1n && a[1]===-9223372036854775808n",
    );
}

#[test]
fn array_like_set_reads_length_once_ignores_iteration_and_observes_prior_writes() {
    check(
        "let a=new Uint8Array(3),log='',source={get [Symbol.iterator](){throw 7;},get length(){log+='l';return {valueOf(){log+='n';return 2.9;}};},get 0(){log+='g0';return {valueOf(){log+='v0';return 7;}};},get 1(){log+='g1';return a[0]+1;}};a.set(source,{valueOf(){log+='o';return 0;}})===undefined && log==='olng0v0g1' && a.join()==='7,8,0'",
    );
    check(
        "let a=Uint8Array.of(1,2,3,4),source={length:3,get 0(){source.length=0;return a[0];},get 1(){return a[1];},get 2(){return a[2];}};a.set(source,1)===undefined && a.join()==='1,1,1,1'",
    );
    check(
        "let a=new Uint8Array(3);a.set('12')===undefined && a.set(7)===undefined && a.set(new DataView(new ArrayBuffer(1)))===undefined && a.join()==='1,2,0'",
    );
    check(
        "let source=Object.create({1:7}),a=new Float64Array(3);source.length=3;a.set(source)===undefined && Number.isNaN(a[0]) && a[1]===7 && Number.isNaN(a[2])",
    );
    exception("new Uint8Array(0).set()", ExceptionKind::TypeError);
    exception("new Uint8Array(0).set(undefined)", ExceptionKind::TypeError);
}

#[test]
fn set_keeps_the_target_length_captured_before_the_array_like_length_getter() {
    check(
        "let buffer=new ArrayBuffer(2,{maxByteLength:6}),a=new Uint8Array(buffer),log='',caught=false;try{a.set({get length(){log+='l';buffer.resize(4);return 3;},get 0(){log+='g';throw 7;}});}catch(e){caught=e instanceof RangeError;}caught && log==='l' && a.length===4 && a[0]===0",
    );
    for constructor_args in ["buffer", "buffer,0,3"] {
        check(&format!(
            "let buffer=new ArrayBuffer(3,{{maxByteLength:6}}),a=new Uint8Array({constructor_args}),log='',source={{get length(){{log+='l';buffer.resize(0);return 3;}},get 0(){{log+='g0';return {{valueOf(){{log+='v0';return 7;}}}};}},get 1(){{log+='g1';buffer.resize(3);return {{valueOf(){{log+='v1';return 8;}}}};}},get 2(){{log+='g2';return {{valueOf(){{log+='v2';return 9;}}}};}}}};a.set(source)===undefined && log==='lg0v0g1v1g2v2' && a.join()==='0,8,9'"
        ));
        check(&format!(
            "let buffer=new ArrayBuffer(3,{{maxByteLength:6}}),a=new Uint8Array({constructor_args}),log='',source={{length:3,0:{{valueOf(){{log+='v0';buffer.resize(0);return 7;}}}},1:{{valueOf(){{log+='v1';buffer.resize(3);return 8;}}}},2:{{valueOf(){{log+='v2';return 9;}}}}}};a.set(source)===undefined && log==='v0v1v2' && a.join()==='0,8,9'"
        ));
    }
}

#[test]
fn array_like_set_converts_after_detachment_and_preserves_effects_before_a_later_throw() {
    check(
        "let a=Uint8Array.of(1,2,3),log='',source={length:3,get 0(){log+='g0';a.buffer.transfer();return {valueOf(){log+='v0';return 7;}};},get 1(){log+='g1';return {valueOf(){log+='v1';return 8;}};},get 2(){log+='g2';return {valueOf(){log+='v2';return 9;}};}};a.set(source)===undefined && log==='g0v0g1v1g2v2' && a.length===0",
    );
    check(
        "let a=Uint8Array.of(1,2,3),log='',caught=false,source={length:3,get 0(){log+='g0';return {valueOf(){log+='v0';return 7;}};},get 1(){log+='g1';return {valueOf(){log+='v1';throw 8;}};},get 2(){log+='g2';throw 9;}};try{a.set(source);}catch(e){caught=e===8;}caught && log==='g0v0g1v1' && a.join()==='7,2,3'",
    );
    check(
        "let a=BigInt64Array.of(1n,2n,3n),moved,log='',caught=false,source={length:3,get 0(){log+='g0';return 7n;},get 1(){log+='g1';moved=a.buffer.transfer();return {valueOf(){log+='v1';return 1;}};},get 2(){log+='g2';throw 7;}};try{a.set(source);}catch(e){caught=e instanceof TypeError;}let saved=new BigInt64Array(moved);caught && log==='g0g1v1' && a.length===0 && saved[0]===7n && saved[1]===2n",
    );
}

#[test]
fn sparse_mutation_checks_byte_offsets_at_the_safe_integer_boundary() {
    check(
        "let maximum=9007199254740991,buffer=new ArrayBuffer(maximum),a=new Uint8Array(buffer,maximum-6,6),guard=new DataView(buffer,maximum-7,1);a.fill(1);a.set([2,3],1);a.reverse();a.copyWithin(0,3,6);buffer.byteLength===maximum && a.join()==='3,2,1,3,2,1' && guard.getUint8(0)===0",
    );
    exception(
        "new Uint8Array(9007199254740991).set({length:9007199254740991},1)",
        ExceptionKind::RangeError,
    );
}

#[test]
fn raw_mutation_avoids_bigint_decoding_while_numeric_reverse_preserves_host_failures() {
    let mut realm = Realm::new(Limits {
        max_bigint_bits: Some(8),
        ..Limits::default()
    });
    assert_eq!(
        realm.eval("let bytes=new Uint8Array(24);for(let i=0;i<24;i++)bytes[i]=255;let source=new BigUint64Array(bytes.buffer),copy=new BigUint64Array(3);source.copyWithin(1,0,2);copy.set(source);let raw=new Uint8Array(copy.buffer);raw.length===24 && raw[0]===255 && raw[23]===255"),
        Ok(Value::Boolean(true))
    );
    realm.eval("let flag=0,little=new Uint8Array(Uint16Array.of(1).buffer)[0]===1,small=new Uint8Array(16),data=new DataView(small.buffer);data.setUint32(little?0:4,1,little);data.setUint32(little?8:12,256,little);let pair=new BigUint64Array(small.buffer)").unwrap();
    assert!(matches!(
        realm.eval("try{pair.reverse();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("data.getUint32(little?0:4,little)===1 && data.getUint32(little?4:0,little)===0 && data.getUint32(little?8:12,little)===256 && data.getUint32(little?12:8,little)===0"),
        Ok(Value::Boolean(true))
    );
    realm.eval("let bounds=0").unwrap();
    assert!(matches!(
        realm.eval(
            "try{pair.fill('256',{valueOf(){bounds++;return 0;}});}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("bounds===0 && flag===0"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn element_counts_are_not_call_argument_quotas() {
    let mut realm = Realm::new(Limits {
        max_arguments: Some(2),
        ..Limits::default()
    });
    assert_eq!(realm.eval("let a=new Uint8Array(5);a.set([1,2,3,4,5]);a.copyWithin(1,0);a.fill(7);a.reverse();a.join()==='7,7,7,7,7'"),Ok(Value::Boolean(true)));
}
